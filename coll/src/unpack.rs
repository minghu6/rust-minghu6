#[macro_export]
macro_rules! impl_unpack {
    ($name:ident| $($trait:ident),+) => {
        $(paste::paste!{ $crate::[<impl_unpack_$trait:snake>]!($name); })+
    };
}


#[macro_export]
macro_rules! impl_unpack_as_ref {
    ($name:ident) => {
        impl<T> AsRef<T> for $name<T> {
            fn as_ref(&self) -> &T {
                &self.0
            }
        }
    };
}


#[macro_export]
macro_rules! impl_unpack_as_mut {
    ($name:ident) => {
        impl<T> AsMut<T> for $name<T> {
            fn as_mut(&mut self) -> &mut T {
                &mut self.0
            }
        }
    };
}


#[macro_export]
macro_rules! impl_unpack_deref {
    ($name:ident) => {
        impl<T> std::ops::Deref for $name<T> {
            type Target = T;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}


#[macro_export]
macro_rules! impl_unpack_deref_mut {
    ($name:ident) => {
        impl<T> std::ops::DerefMut for $name<T> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}


#[macro_export]
macro_rules! impl_unpack_from {
    ($name:ident) => {
        impl<T> From<T> for $name<T> {
            fn from(other: T) -> Self {
                Self(other)
            }
        }
    };
}
