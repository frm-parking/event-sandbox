pub trait IsError {
	fn is_error(&self) -> bool;
}

pub macro residuality($err:tt to $variant:tt for $state:tt) {
	impl $crate::err::IsError for $state {
		fn is_error(&self) -> bool {
			::core::matches!(self, $state::$variant(_))
		}
	}

	impl ::std::convert::From<$err> for $state {
		fn from(value: $err) -> Self {
			Self::$variant(value)
		}
	}

	impl<T, E> ::std::ops::FromResidual<::std::result::Result<T, E>> for $state
	where
		E: ::std::convert::Into<$state>,
	{
		fn from_residual(residual: ::std::result::Result<T, E>) -> Self {
			if let Err(err) = residual {
				err.into()
			} else {
				unreachable!()
			}
		}
	}
}
