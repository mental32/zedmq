use super::builder::{IsARealSocketType, SocketBuilder};

impl IsARealSocketType for RouterType {
    fn construct_from(_: &()) -> Result<Self, ()> {
        Err(())
    }

    fn check_build_information(_: &SocketBuilder<Self>) -> Result<(), ()> {
        Err(())
    }
}

pub struct RouterType;
