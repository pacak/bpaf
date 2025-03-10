use std::future::Future;

use crate::executor::PoisonHandle;

enum Action<'a> {
    Fut(Box<dyn Future<Output = PoisonHandle> + 'a>),
}
