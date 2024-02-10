//! `turbostate` is a library for building state machines in Rust.
//!
//! # Overview
//!
//! `turbostate` provides abstractions and utilities to define and manage state machines.
//! It offers a generic `Machine` struct that encapsulates the state and shared data of
//! a state machine, and provides methods to advance the state based on events.
//!
//! # Usage
//!
//! To use `turbostate`, you typically define your state machine by implementing the `Engine` trait.
//! This trait requires you to define the types representing states, events, errors, and shared data
//! of your state machine, as well as the logic to transition between states based on events.

#![feature(decl_macro)]
#![feature(try_trait_v2)]

use std::marker::PhantomData;
use std::ops::FromResidual;
use std::sync::Arc;
use std::sync::Mutex;

/// `Flow` represents the possible outcomes of state transitions in the state machine.
#[derive(Debug)]
pub enum Flow<T, E, B> {
	/// Skip to the next step without changing the state.
	Pass,
	/// Transition to a new state.
	Transition(T),
	/// Jump to another branch within the same event, specifying a new state and event.
	Slide(T, B),
	/// Raise an error if an error occurs during the transition.
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

/// `Engine` is a trait that must be implemented on the state machine.
///
/// This trait defines the behavior of the state machine, including state transitions
/// based on events, error handling, and shared data management.
pub trait Engine {
	/// Represents the enum of possible states.
	type State;
	/// Represents the type of events that drive state transitions.
	type Event;
	/// Represents the type of errors that can occur during state transitions.
	type Error;
	/// Represents any shared data that is accessible by all states.
	type Shared;

	/// Advances the state machine based on the current state, incoming event, and shared data.
	#[cfg(feature = "async")]
	#[allow(unused)]
	async fn next(
		state: &Self::State,
		event: Self::Event,
		shared: &mut Self::Shared,
	) -> Flow<Self::State, Self::Error, Self::Event> {
		Flow::Pass
	}

	/// Advances the state machine based on the current state, incoming event, and shared data.
	#[cfg(not(feature = "async"))]
	#[allow(unused)]
	fn next(
		state: &Self::State,
		event: Self::Event,
		shared: &mut Self::Shared,
	) -> Flow<Self::State, Self::Error, Self::Event> {
		Flow::Pass
	}
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

/// `Machine` is a struct that encapsulates the state and shared data of the state machine,
/// providing methods to advance the state based on events.
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
	/// Creates a new `Machine` with the initial state and default shared data.
	pub fn new(initial: E::State) -> Self
	where
		E::Shared: Default,
	{
		Self {
			store: Arc::new(Store::new(initial, E::Shared::default())),
			event: Default::default(),
		}
	}

	/// Creates a new `Machine` with the initial state and specified shared data.
	pub fn new_shared(initial: E::State, shared: E::Shared) -> Self {
		Self {
			store: Arc::new(Store::new(initial, shared)),
			event: Default::default(),
		}
	}

	/// Creates a new `Machine` with the default state and specified shared data.
	pub fn default_shared(shared: E::Shared) -> Self
	where
		E::State: Default,
	{
		Self {
			store: Arc::new(Store::new(E::State::default(), shared)),
			event: Default::default(),
		}
	}

	fn set_state(&self, new_state: E::State) {
		let mut state = self.store.state.lock().unwrap();
		*state = new_state;
	}

	fn infer_result(&self, flow: Flow<E::State, E::Error, E::Event>) -> Result<(), E::Error> {
		match flow {
			Flow::Pass => Ok(()),
			Flow::Transition(new_state) => {
				self.set_state(new_state);
				Ok(())
			}
			Flow::Slide(new_state, event) => {
				self.set_state(new_state);
				self.fire(event)
			}
			Flow::Failure(err) => Err(err),
		}
	}

	/// Fires the specified event on the state machine to advance the state asynchronously.
	#[cfg(feature = "async")]
	pub async fn fire(&self, event: E::Event) -> Result<(), E::Error> {
		self.infer_result({
			let state = self.store.state.lock().unwrap();
			let mut shared = self.store.shared.lock().unwrap();
			E::next(&state, event, &mut shared).await
		})
	}

	/// Fires the specified event on the state machine to advance the state.
	#[cfg(not(feature = "async"))]
	pub fn fire(&self, event: E::Event) -> Result<(), E::Error> {
		self.infer_result({
			let state = self.store.state.lock().unwrap();
			let mut shared = self.store.shared.lock().unwrap();
			E::next(&state, event, &mut shared)
		})
	}
}
