use super::executor::*;

struct App<E = DefaultExecutor> 
    where E: Executor
{
    exec: E, 
    world: World,
}

impl<E> App<E> 
    where E: Executor
{
    fn builder() -> AppBuilder {
        AppBuilder {

        }
    }
}

struct AppBuilder {
    
}

impl AppBuilder {

}

