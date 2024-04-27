use crate::handle::Handle;
use crate::method::BootstrapMethodArgument;

pub struct ConstantDynamic<'a> {
    name: &'a str,
    descriptor: &'a str,
    bootstrap_method: &'a Handle,
    bootstrap_method_arguments: &'a [&'a BootstrapMethodArgument<'a>],
}
