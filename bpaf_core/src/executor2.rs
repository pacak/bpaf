use std::future::Future;

use crate::executor::futures::ErrorHandle;

enum Action<'a> {
    Fut(Box<dyn Future<Output = ErrorHandle> + 'a>),
}
