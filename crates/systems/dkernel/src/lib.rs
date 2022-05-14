mod audit;
mod error;
mod hirs;
mod history;
pub mod query_result;
pub mod repository;
mod state;

use std::{any::TypeId, collections::HashMap};

use audit::audit;
use bevy_ecs::prelude::Component;
use components::{event::Event, rules::AuditResponse, snapshot::Snapshot};
use hirs::Hirs;
use history::History;
use repository::Repository;
use state::State;

#[derive(Component)]
pub struct Kernel {
    repository: Box<dyn Repository + Send + Sync + 'static>,
    hirs: Hirs,
    pub snapshot: Snapshot,
    history: History,
    states: HashMap<TypeId, Box<dyn State + Send + Sync + 'static>>,
}

impl Kernel {
    pub fn new(repository: impl Repository + Send + Sync + 'static) -> Self {
        Self {
            repository: Box::new(repository),
            hirs: Default::default(),
            snapshot: Default::default(),
            history: Default::default(),
            states: Default::default(),
        }
    }

    pub fn commit(&mut self, event: Event) {
        self.repository.commit(event);
    }

    pub fn process(&mut self) {
        let entries = self.repository.poll();
        for entry in entries {
            if audit(&self.snapshot, &entry) == AuditResponse::Allowed {
                self.hirs.handle_event(&entry.event);
                self.history.handle_event(&self.snapshot, &entry.event);
                for state in self.states.values_mut() {
                    state.handle_event(&self.snapshot, &entry.event);
                }
                self.snapshot.handle_event(&entry.event);
            }
        }
    }

    pub fn add_state<T: State + Send + Sync + 'static>(&mut self, state: T) {
        self.states.insert(TypeId::of::<T>(), Box::new(state));
    }

    pub fn get_state<T: State + Send + Sync + 'static>(&self) -> Option<&T> {
        self.states
            .get(&TypeId::of::<T>())
            .map(|state| state.as_any().downcast_ref::<T>().unwrap())
    }

    pub fn get_state_mut<T: State + Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.states
            .get_mut(&TypeId::of::<T>())
            .map(|state| state.as_any_mut().downcast_mut::<T>().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use components::event::{Event, EventEntry};
    use components::patch::FilePatch;
    use components::rules::{NodeOperation, Rules};
    use components::user::UserId;
    use components::{content::Content, patch::ChildrenPatch};
    use deskc_hir::expr::Literal;
    use deskc_hir::helper::remove_meta;
    use deskc_hir::ty::Type as HirType;
    use deskc_hir::{
        expr::Expr,
        meta::{Meta, WithMeta},
    };
    use deskc_ids::{CardId, FileId, IrId, LinkName, NodeId};
    use deskc_types::Type;
    use hirs::HirQueries;

    use super::*;

    #[mry::mry]
    #[derive(Default)]
    pub struct TestRepository {}

    #[mry::mry]
    impl Repository for TestRepository {
        fn poll(&mut self) -> Vec<EventEntry> {
            panic!()
        }
        fn commit(&mut self, log: Event) {
            panic!()
        }
        fn add_owner(&mut self, user_id: UserId) {
            panic!()
        }
        fn remove_owner(&mut self, user_id: UserId) {
            panic!()
        }
    }

    #[mry::mry]
    #[derive(Default)]
    pub struct TestState {}

    #[mry::mry]
    impl State for TestState {
        fn handle_event(&mut self, _snapshot: &Snapshot, _: &Event) {
            panic!()
        }
    }

    #[test]
    fn integration() {
        let mut repository = TestRepository::default();

        let user_a = UserId("a".into());
        let user_b = UserId("b".into());
        let node_a = NodeId::new();
        let node_b = NodeId::new();
        let file_id = FileId::new();

        repository.mock_poll().returns(vec![
            EventEntry {
                index: 0,
                user_id: user_a.clone(),
                event: Event::AddOwner {
                    user_id: user_a.clone(),
                },
            },
            EventEntry {
                index: 0,
                user_id: user_a.clone(),
                event: Event::AddFile(file_id.clone()),
            },
            EventEntry {
                index: 0,
                user_id: user_a.clone(),
                event: Event::PatchFile {
                    file_id: file_id.clone(),
                    patch: FilePatch::UpdateRules {
                        rules: Rules {
                            default: [NodeOperation::AddNode, NodeOperation::PatchChildrenInsert]
                                .into_iter()
                                .collect(),
                            users: Default::default(),
                        },
                    },
                },
            },
            EventEntry {
                index: 0,
                user_id: user_b.clone(),
                event: Event::AddOwner {
                    user_id: user_b.clone(),
                },
            },
            EventEntry {
                index: 0,
                user_id: user_a.clone(),
                event: Event::AddNode {
                    node_id: node_a.clone(),
                    file_id: file_id.clone(),
                    content: Content::Apply {
                        ty: Type::Function {
                            parameters: vec![Type::String],
                            body: Box::new(Type::Number),
                        },
                        link_name: Default::default(),
                    },
                },
            },
            EventEntry {
                index: 1,
                user_id: user_b.clone(),
                event: Event::AddNode {
                    node_id: node_b.clone(),
                    file_id,
                    content: Content::String("string".into()),
                },
            },
            EventEntry {
                index: 1,
                user_id: user_b,
                event: Event::PatchChildren {
                    node_id: node_a.clone(),
                    patch: ChildrenPatch::Insert {
                        index: 0,
                        node: node_b,
                    },
                },
            },
        ]);

        let mut test_state = TestState::default();
        test_state.mock_handle_event(mry::Any, mry::Any).returns(());

        let mut kernel = Kernel::new(repository);
        kernel.add_state(test_state);
        kernel.process();

        assert_eq!(kernel.snapshot.flat_nodes.len(), 2);
        assert_eq!(kernel.snapshot.owners.len(), 1);
        assert_eq!(
            remove_meta(kernel.hirs.hir(node_a).unwrap().as_ref().clone()),
            WithMeta {
                id: IrId::default(),
                meta: Meta::default(),
                value: Expr::Apply {
                    function: WithMeta {
                        id: IrId::default(),
                        meta: Meta::default(),
                        value: HirType::Function {
                            parameters: vec![WithMeta {
                                id: IrId::default(),
                                meta: Meta::default(),
                                value: HirType::String
                            }],
                            body: Box::new(WithMeta {
                                id: IrId::default(),
                                meta: Meta::default(),
                                value: HirType::Number
                            }),
                        }
                    },
                    link_name: LinkName::None,
                    arguments: vec![WithMeta {
                        id: IrId::default(),
                        meta: Meta::default(),
                        value: Expr::Literal(Literal::String("string".into()))
                    }]
                }
            }
        );

        kernel
            .get_state_mut::<TestState>()
            .unwrap()
            .mock_handle_event(mry::Any, mry::Any)
            .assert_called(6);

        // asserts handle_event was called with unprocessed snapshot
        kernel
            .get_state_mut::<TestState>()
            .unwrap()
            .mock_handle_event(Snapshot::default(), Event::AddOwner { user_id: user_a })
            .assert_called(1);
    }
}
