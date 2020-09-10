use core::marker::PhantomData;

pub  trait IsARealSocketType: Sized {
    fn constructor(state: &()) -> Result<Self, ()>;

    fn check_build_information(builder: &SocketBuilder<Self>) -> Result<(), ()>;
}

pub struct SocketBuilder<S: IsARealSocketType> {
    _raw_type: PhantomData<S>,
}

impl<S: IsARealSocketType> SocketBuilder<S> {
    pub(crate) fn new() -> Self {
        Self {
            _raw_type: PhantomData,
        }
    }

    pub async fn build(self) -> Result<S, ()> {
        S::check_build_information(&self)?;

        // Do build stuff
        // i.e. tcp connect/listen

        // S::constructor(build_state)
        Err(())
    }
}
