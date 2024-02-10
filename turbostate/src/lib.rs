#![feature(decl_macro)]
#![feature(try_trait_v2)]

use std::marker::PhantomData;
use std::ops::FromResidual;
use std::sync::Arc;
use std::sync::Mutex;

pub mod err;

#[derive(Debug)]
pub enum Flow<T, E, B> {
	Pass,
	Transition(T),
	Branch(T, B),
	Failure(E),
}

impl<T, E, B, X, R> FromResidual<Result<T, E>> for Flow<X, R, B>
where
	E: Into<R>,
{
	fn from_residual(residual: Result<T, E>) -> Self {
		if let Err(err) = residual {
			Self::Failure(err.into())
		} else {
			unreachable!()
		}
	}
}

pub trait Engine {
	type State;
	type Event;
	type Error;
	type Shared;

	fn flow(
		state: &Self::State,
		event: Self::Event,
		shared: &mut Self::Shared,
	) -> Flow<Self::State, Self::Error, Self::Event>;
}

#[derive(Debug, Default)]
struct Store<T, S> {
	state: Mutex<T>,
	shared: Mutex<S>,
}

impl<T, S> Store<T, S> {
	pub fn new(state: T, shared: S) -> Self {
		Self {
			state: Mutex::new(state),
			shared: Mutex::new(shared),
		}
	}
}

#[derive(Debug, Clone)]
pub struct Machine<E: Engine> {
	store: Arc<Store<E::State, E::Shared>>,
	event: PhantomData<E::Event>,
}

impl<E: Engine> Default for Machine<E>
where
	E::State: Default,
	E::Shared: Default,
{
	fn default() -> Self {
		Self {
			store: Default::default(),
			event: Default::default(),
		}
	}
}

impl<E: Engine> Machine<E> {
	pub fn new(initial: E::State) -> Self
	where
		E::Shared: Default,
	{
		Self {
			store: Arc::new(Store::new(initial, E::Shared::default())),
			event: Default::default(),
		}
	}

	pub fn new_shared(initial: E::State, shared: E::Shared) -> Self {
		Self {
			store: Arc::new(Store::new(initial, shared)),
			event: Default::default(),
		}
	}

	pub fn default_shared(shared: E::Shared) -> Self
	where
		E::State: Default,
	{
		Self {
			store: Arc::new(Store::new(E::State::default(), shared)),
			event: Default::default(),
		}
	}

	pub fn fire(&self, event: E::Event) -> Result<(), E::Error> {
		let result = {
			let state = self.store.state.lock().unwrap();
			let mut shared = self.store.shared.lock().unwrap();
			E::flow(&state, event, &mut shared)
		};

		match result {
			Flow::Pass => Ok(()),
			Flow::Transition(new_state) => {
				let mut state = self.store.state.lock().unwrap();
				*state = new_state;
				Ok(())
			}
			Flow::Branch(new_state, event) => {
				{
					let mut state = self.store.state.lock().unwrap();
					*state = new_state;
				}

				self.fire(event)
			}
			Flow::Failure(err) => Err(err),
		}
	}
}
