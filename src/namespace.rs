use std::{cell::RefCell, marker::PhantomData, rc::Rc};

trait NamespaceStateSealed {}
#[expect(private_bounds)]
pub trait NamespaceState: NamespaceStateSealed {}

pub enum NamespaceOpen {}
impl NamespaceStateSealed for NamespaceOpen {}
impl NamespaceState for NamespaceOpen {}

pub enum NamespaceFinalized {}
impl NamespaceStateSealed for NamespaceFinalized {}
impl NamespaceState for NamespaceFinalized {}

pub struct Namespace<S: NamespaceState> {
    names: Vec<NameInner>,
    state: PhantomData<S>,
}

impl Namespace<NamespaceOpen> {
    pub fn insert(&mut self, s: String) -> Name {
        todo!()
    }

    pub fn finalize(self) -> Namespace<NamespaceFinalized> {
        let Self { names, state } = self;
        Namespace {
            names,
            state: PhantomData,
        }
    }
}

enum NameInner {
    Pending,
    Resolved(String),
}

impl NameInner {
    fn as_resolved(&self) -> String {
        let NameInner::Resolved(s) = self else {
            panic!("The Namespace to which this belongs has not been finalized")
        };
        s.clone()
    }
}

pub struct Name {
    inner: Rc<RefCell<NameInner>>,
}

impl Name {
    pub fn to_string(&self) -> String {
        self.inner.try_borrow().unwrap().as_resolved()
    }
}
