use super::builder::{IsARealSocketType, SocketBuilder};

impl IsARealSocketType for RouterType {
    fn constructor(_: &()) -> Result<Self, ()> { Err(()) }

    fn check_build_information(_: &SocketBuilder<Self>) -> Result<(), ()> { Err(()) }
}

pub struct RouterType;
