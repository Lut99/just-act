//  SIMULATION.rs
//    by Lut99
//
//  Created:
//    16 Apr 2024, 11:06:51
//  Last edited:
//    21 May 2024, 15:12:22
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the main simulation loop that can run agents.
//

use std::any::type_name;
use std::cell::RefCell;
use std::collections::HashSet;
use std::convert::Infallible;
use std::error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::rc::Rc;

use console::Style;
use justact_core::agent::{AgentPoll, RationalAgent};
use justact_core::auxillary::Identifiable;
use justact_core::ExtractablePolicy;
use log::{debug, info};
use stackvec::StackVec;

use crate::global::{GlobalState, GlobalView, Timestamp};
use crate::interface::Interface;
use crate::local::{LocalState, LocalView, Target};
use crate::sync::Synchronizer;
use crate::wire::{Action, Agreement, Message};


/***** ERROR *****/
/// Defines errors originating in the [`Simulation`].
#[derive(Debug)]
pub enum Error<E> {
    /// Some agent errored.
    AgentPoll { agent: &'static str, err: E },
}
impl<E: Display> Display for Error<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            AgentPoll { agent, .. } => write!(f, "Failed to poll agent {agent}"),
        }
    }
}
impl<E: 'static + error::Error> error::Error for Error<E> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            AgentPoll { err, .. } => Some(err),
        }
    }
}





/***** LIBRARY *****/
/// Runs a simulation with the given agents.
///
/// The simulation runs until all given agents are dead.
///
/// # Generics
/// - `S1`: The [`Synchronizer`] used to reach consensus on new agreements.
/// - `S2`: The [`Synchronizer`] used to reach consensus on new times.
/// - `A`: Some generic kind over the specific [`Agent`] required for this implementation. It is recommended to make some sum Agent type yourself that abstracts over the different ones if necessary.
#[derive(Debug)]
pub struct Simulation<S1, S2, A> {
    /// The [`GlobalState`] that agents communicate synchronized information through.
    global:    GlobalState<S1, S2>,
    /// The [`LocalState`] that agents communicate potentially asynchronized information through.
    local:     LocalState,
    /// The (alive!) agents in the simulation.
    agents:    Vec<A>,
    /// A set of action (identifiers) of the ones we've already audited
    audited:   HashSet<&'static str>,
    /// An interface we use to log whatever happens in pretty ways.
    interface: Rc<RefCell<Interface>>,
}
impl<S1, S2, A> Simulation<S1, S2, A> {
    /// Creates a new Simulation with no agents registered yet.
    ///
    /// # Arguments
    /// - `sync1`: The [`Synchronizer`] used to reach consensus on new [`Agreements`](crate::global::Agreements).
    /// - `sync2`: The [`Synchronizer`] used to reach consensus on new [`Times`](crate::global::Times).
    ///
    /// # Returns
    /// An empty simulation that wouldn't run anything.
    #[inline]
    pub fn new(sync1: S1, sync2: S2) -> Self {
        info!("Creating demo Simulation<{}, {}, {}>", type_name::<S1>(), type_name::<S2>(), type_name::<A>());

        // Build an interface with ourselves registered
        let mut interface: Interface = Interface::new();
        interface.register("<system>", Style::new().bold());

        // Create ourselves with that
        let interface: Rc<RefCell<Interface>> = Rc::new(RefCell::new(interface));
        Self {
            global: GlobalState::new(Timestamp(0), sync1, sync2, interface.clone()),
            local: LocalState::new(interface.clone()),
            agents: Vec::new(),
            audited: HashSet::new(),
            interface,
        }
    }

    /// Creates a new Simulation with no agents registered yet, but space to do so before re-allocation is triggered.
    ///
    /// # Arguments
    /// - `sync1`: The [`Synchronizer`] used to reach consensus on new [`Agreements`](crate::global::Agreements).
    /// - `sync2`: The [`Synchronizer`] used to reach consensus on new [`Times`](crate::global::Times).
    /// - `capacity`: The number of agents for which there should _at least_ be space.
    ///
    /// # Returns
    /// An empty simulation that wouldn't run anything but that has space for at least `capacity` agents.
    #[inline]
    pub fn with_capacity(sync1: S1, sync2: S2, capacity: usize) -> Self {
        info!("Creating demo Simulation<{}, {}, {}> (with capacity '{}')", type_name::<S1>(), type_name::<S2>(), type_name::<A>(), capacity);

        // Build an interface with ourselves registered
        let mut interface: Interface = Interface::new();
        interface.register("<system>", Style::new().bold());

        // Create ourselves with that
        let interface: Rc<RefCell<Interface>> = Rc::new(RefCell::new(interface));
        Self {
            global: GlobalState::new(Timestamp(0), sync1, sync2, interface.clone()),
            local: LocalState::new(interface.clone()),
            agents: Vec::with_capacity(capacity),
            audited: HashSet::new(),
            interface,
        }
    }

    /// Builds a new Simulation with the given set of agents registered to it from the get-go.
    ///
    /// # Arguments
    /// - `sync1`: The [`Synchronizer`] used to reach consensus on new [`Agreements`](crate::global::Agreements).
    /// - `sync2`: The [`Synchronizer`] used to reach consensus on new [`Times`](crate::global::Times).
    /// - `agents`: Some list of `A`gents that should be registered right away.
    ///
    /// # Returns
    /// A Simulation with the given `agents` already registered in it.
    #[inline]
    pub fn with_agents(sync1: S1, sync2: S2, agents: impl IntoIterator<Item = A>) -> Self {
        info!("Creating demo Simulation<{}, {}, {}> with agents", type_name::<S1>(), type_name::<S2>(), type_name::<A>());

        // Build an interface with ourselves registered
        let mut interface: Interface = Interface::new();
        interface.register("<system>", Style::new().bold());

        // Create agents out of the given iterator, logging as we go
        let agents: Vec<A> = agents
            .into_iter()
            .enumerate()
            .map(|(i, a)| {
                debug!("Registered agent {}", i);
                a
            })
            .collect();

        // Now built self
        let interface: Rc<RefCell<Interface>> = Rc::new(RefCell::new(interface));
        Self {
            global: GlobalState::new(Timestamp(0), sync1, sync2, interface.clone()),
            local: LocalState::new(interface.clone()),
            agents,
            audited: HashSet::new(),
            interface,
        }
    }
}
impl<S1, S2, A: Identifiable<Id = &'static str>> Simulation<S1, S2, A> {
    /// Registers a new agent after creation.
    ///
    /// # Arguments
    /// - `agent`: The new `A`gent to register.
    /// - `style`: A [`Style`] that is used to format the agent's ID during logging.
    #[inline]
    pub fn register(&mut self, agent: impl Into<A>, style: Style) {
        debug!("Registered agent {}", self.agents.len());

        // Register the agent in the interface
        let agent: A = agent.into();
        self.interface.borrow_mut().register(agent.id(), style);

        // Put it in the simulation
        self.agents.push(agent.into());
    }
}
impl<S1, S2, A> Simulation<S1, S2, A>
where
    S1: Synchronizer<Agreement>,
    S1::Error: 'static,
    S2: Synchronizer<Timestamp>,
    S2::Error: 'static,
    A: Identifiable<Id = &'static str>,
    A: RationalAgent<Enactment = Action, Action = Action, Statement = Message, Message = Message, Target = Target, Error = Infallible>,
{
    /// Polls all the agents in the simulation once.
    ///
    /// # Returns
    /// True if at least one agent is alive, or false otherwise.
    ///
    /// # Errors
    /// This function errors if any of the agents fails to communicate with the end-user or other agents.
    pub fn poll(&mut self) -> Result<bool, Error<A::Error>> {
        info!("Starting new agent iteration");

        // Iterate over the agents and only keep those that report they wanna be kept
        let mut agents: StackVec<64, A> = StackVec::new();
        for (i, mut agent) in self.agents.drain(..).enumerate() {
            debug!("Polling agent {}...", i);

            // Prepare calling the agent's poll method
            let id: &'static str = agent.id();
            let mut global: GlobalView<S1, S2> = self.global.scope(id);
            let mut local: LocalView = self.local.scope(id);

            // Do the call then
            match agent.poll(&mut global, &mut local) {
                Ok(AgentPoll::Alive) => agents.push(agent),
                Ok(AgentPoll::Dead) => continue,
                Err(err) => return Err(Error::AgentPoll { agent: id, err }),
            }
        }

        // Now re-instante those kept and return whether we're done
        self.agents.extend(agents);
        Ok(!self.agents.is_empty())
    }

    /// Runs the simulation until no more agents are alive.
    ///
    /// # Errors
    /// This function errors if any of the agents fails to communicate with the end-user or other agents.
    #[inline]
    pub fn run<P>(&mut self) -> Result<(), Error<A::Error>>
    where
        for<'m> P: ExtractablePolicy<&'m Message>,
    {
        loop {
            // Run the next iteration
            let reiterate: bool = self.poll()?;

            // Run an audit
            debug!("Running audit on {} actions...", self.local.acts.acts.len());
            while let Err(expl) = self.local.acts.audit::<S1, P>(&self.global.agrmnts.unspecific(), &self.local.stmts.full(), Some(&mut self.audited))
            {
                // Write the problem
                self.interface.borrow().error_audit("<system>", expl);
            }

            // Stop if no agents are alive
            if !reiterate {
                return Ok(());
            }
        }
    }
}
