pub enum MyExitCode { OK, FAIL }
pub trait MyTermination {
    fn report(self) -> MyExitCode;
}

impl<T> MyTermination for Option<T> {
    fn report(self) -> MyExitCode {
        match self {
            Some(_) => MyExitCode::OK,
            None    => MyExitCode::FAIL,
        }
    }
}

impl<T,E> MyTermination for Result<T,E> {
    fn report(self) -> MyExitCode {
        match self {
            Ok(_)  => MyExitCode::OK,
            Err(_)=> MyExitCode::FAIL,
        }
    }
}

impl MyTermination for () {
    fn report(self) -> MyExitCode {
        MyExitCode::OK
    }
}

pub fn trampoline<T: MyTermination>(main: fn()->T) -> Result<(),()> {
    match main().report() {
        MyExitCode::OK   => Ok(()),
        MyExitCode::FAIL => Err(()),
    }
}
