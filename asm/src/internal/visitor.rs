#[macro_export]
macro_rules! asm_visitor {
    (
        $(#[$classMeta:meta])*
        $vis:vis struct $visitorName:ident<$lt:lifetime>
    ) => {
        $(#[$classMeta])*
        #[derive(Default)]
        $vis struct $visitorName<$lt> {
            $vis delegated: Option<&$lt $visitorName<$lt>>,
        }
        
        #[allow(dead_code)]
        impl<$lt> $visitorName<$lt> {
            pub fn new() -> Self {
                Default::default()
            }
            pub fn from(origin: &$lt $visitorName) -> Self {
                Self {
                    delegated: Some(origin)
                }
            }
        }
    };
}

#[macro_export]
macro_rules! asm_visitor_impl {
    (
        $(#[$classMeta:meta])*
        impl $visitorName:ident<$lt:lifetime> {
            $(
                $(#[$funMeta:meta])*
                $vis:vis fn $innerFuncName:ident(
                    &self $(,)? $($parameterName:ident: $parameterType: ty),*
                ) -> Option<$returns:ty>;
            )*
        }
    ) => {
        $(#[$classMeta])*
        #[allow(dead_code)]
        impl $visitorName<$lt> {
            $(
                $(#[$funMeta])*
                $vis fn $innerFuncName(&self, $($parameterName: $parameterType),*) -> Option<$returns> {
                    self.delegated?.$innerFuncName($($parameterName),*)
                }
            )*
        }
    };
}
