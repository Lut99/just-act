//  SIMULATION.rs
//    by Lut99
//
//  Created:
//    16 Apr 2024, 11:06:51
//  Last edited:
//    18 Apr 2024, 17:25:59
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the main simulation loop that can run agents.
//

use std::error;
use std::fmt::{Display, Formatter, Result as FResult};

use console::Style;
use justact_core::agent::{Agent, AgentPoll, RationalAgent};
use justact_core::statements::Statements as _;
use log::{debug, info};
use stackvec::StackVec;

use crate::interface::Interface;
use crate::statements::{Scope, Statements};


/***** ERROR *****/
/// Defines errors originating in the [`Simulation`].
#[derive(Debug)]
pub enum Error<E> {
    /// Some agent errored.
    AgentPoll { i: usize, err: E },
}
impl<E: Display> Display for Error<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            AgentPoll { i, .. } => write!(f, "Failed to poll agent {i}"),
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
/// - `A`: Some generic kind over the specific [`Agent`] required for this implementation. It is recommended to make some sum Agent type yourself that abstracts over the different ones if necessary.
#[derive(Debug)]
pub struct Simulation<A> {
    /// The [`MessagePool`] that agents communicate through.
    stmts:     Statements,
    /// The [`Interface`] that agents report through.
    interface: Interface,
    /// The (alive!) agents in the simulation.
    agents:    Vec<A>,
}
impl<A> Default for Simulation<A> {
    #[inline]
    fn default() -> Self { Self::new() }
}
impl<A> Simulation<A> {
    /// Creates a new Simulation with no agents registered yet.
    ///
    /// # Returns
    /// An empty simulation that wouldn't run anything.
    #[inline]
    pub fn new() -> Self {
        info!("Creating demo Simulation");

        // Create an interface with ourselves in it
        let mut interface: Interface = Interface::new();
        interface.register("<system>", Style::new().bold());

        // Create ourselves with that
        Self { stmts: Statements::new(), interface, agents: Vec::new() }
    }

    /// Creates a new Simulation with no agents registered yet, but space to do so before re-allocation is triggered.
    ///
    /// # Arguments
    /// - `capacity`: The number of agents for which there should _at least_ be space.
    ///
    /// # Returns
    /// An empty simulation that wouldn't run anything but that has space for at least `capacity` agents.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        info!("Creating demo Simulation");

        // Create an interface with ourselves in it
        let mut interface: Interface = Interface::new();
        interface.register("<system>", Style::new().bold());

        // Create ourselves with that
        Self { stmts: Statements::new(), interface, agents: Vec::with_capacity(capacity) }
    }

    /// Builds a new Simulation with the given set of agents registered to it from the get-go.
    ///
    /// # Arguments
    /// - `agents`: Some list of `A`gents that should be registered right away.
    ///
    /// # Returns
    /// A Simulation with the given `agents` already registered in it.
    #[inline]
    pub fn with_agents(agents: impl IntoIterator<Item = A>) -> Self {
        info!("Creating demo Simulation");

        // Create an interface with ourselves in it
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
        debug!("Created demo Simulation with {} agents", agents.len());
        Self { stmts: Statements::new(), interface, agents }
    }

    /// Registers a new agent after creation.
    ///
    /// # Arguments
    /// - `agent`: The new `A`gent to register.
    #[inline]
    pub fn register(&mut self, agent: impl Into<A>) {
        debug!("Registered agent {}", self.agents.len());
        self.agents.push(agent.into());
    }

    /// Registers a new agent after creation, calling the provided constructor for it.
    ///
    /// # Arguments
    /// - `constructor_fn`: Some constructor to create an Agent that is compatible with `A`. It should accept a mutable reference to an [`Interface`] to register itself.
    #[inline]
    pub fn register_with_interface<APrime: Into<A>>(&mut self, constructor_fn: impl FnOnce(&mut Interface) -> APrime) {
        debug!("Registered agent {}", self.agents.len());
        self.agents.push(constructor_fn(&mut self.interface).into());
    }

    /// Returns a reference to the internal [`Statements`].
    #[inline]
    pub fn stmts(&self) -> &Statements { &self.stmts }

    /// Returns a mutable reference to the internal [`Statements`].
    #[inline]
    pub fn stmts_mut(&mut self) -> &mut Statements { &mut self.stmts }

    /// Returns a reference to the internal [`Interface`].
    #[inline]
    pub fn interface(&self) -> &Interface { &self.interface }

    /// Returns a mutable reference to the internal [`Interface`].
    #[inline]
    pub fn interface_mut(&mut self) -> &mut Interface { &mut self.interface }
}
impl<A> Simulation<A>
where
    A: Agent<Identifier = &'static str> + RationalAgent<Statements = Statements, Interface = Interface>,
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
        let mut agents: StackVec<64, A> = StackVec::new();
        for (i, mut agent) in self.agents.drain(..).enumerate() {
            debug!("Polling agent {}...", i);
            let id: &'static str = agent.id();
            match agent.poll(&mut self.stmts.scope(Scope::Agent(id)), &mut self.interface) {
                Ok(AgentPoll::Alive) => agents.push(agent),
                Ok(AgentPoll::Dead) => continue,
                Err(err) => return Err(Error::AgentPoll { i, err }),
            }
        }
        self.agents.extend(agents);
        Ok(!self.agents.is_empty())
    }

    /// Runs the simulation until no more agents are alive.
    ///
    /// # Errors
    /// This function errors if any of the agents fails to communicate with the end-user or other agents.
    #[inline]
    pub fn run(&mut self) -> Result<(), Error<A::Error>> {
        loop {
            // Run the next iteration
            let reiterate: bool = self.poll()?;

            // Run an audit
            if self.stmts.any_enacted() {
                debug!("Running audit on {} actions...", self.stmts.n_enacted());
                match self.stmts.audit() {
                    Ok(_) => self.interface.log("<system>", "All actions are valid"),
                    Err(expl) => self.interface.error_audit_datalog("<system>", expl),
                }
            }

            // Stop if no agents are alive
            if !reiterate {
                return Ok(());
            }
        }
    }
}
