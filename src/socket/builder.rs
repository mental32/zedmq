use core::marker::PhantomData;

use crate::context::Context;

pub trait IsARealSocketType: Sized {
    fn construct_from(state: &()) -> Result<Self, ()>;

    fn check_build_information(builder: &SocketBuilder<Self>) -> Result<(), ()>;
}

enum Position<'a> {
    Connect(&'a str),
    Bind(&'a str),
}

pub struct SocketBuilder<'a, S: IsARealSocketType> {
    _raw_type: PhantomData<S>,
    position: Option<Position<'a>>,
}

impl<'a, S: IsARealSocketType> SocketBuilder<'a, S> {
    pub(crate) fn new() -> Self {
        Self {
            _raw_type: PhantomData,
            position: None,
        }
    }

    pub fn within(self, context: &Context) -> Self {
        self
    }

    pub fn bind(mut self, address: &'a str) -> Self {
        self.position = Some(Position::Bind(address));
        self
    }

    pub fn connect(mut self, address: &'a str) -> Self {
        self.position = Some(Position::Connect(address));
        self
    }

    pub async fn build(self) -> Result<S, ()> {
        // Do build stuff

        S::check_build_information(&self)?;

        // i.e. tcp connect/listen
        match self.position {
            None => return Err(()),
            Some(Position::Connect(endpoint)) => (),
            Some(Position::Bind(endpoint)) => (),
        }

        // S::construct_from(build_state)
        Err(())
    }
}
