//  SIMULATION.rs
//    by Lut99
//
//  Created:
//    16 Apr 2024, 11:06:51
//  Last edited:
//    17 Apr 2024, 11:25:01
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the main simulation loop that can run agents.
//

use std::error;
use std::fmt::{Display, Formatter, Result as FResult};

use console::Style;
use justact_core::agent::{AgentPoll, RationalAgent};
use justact_core::collection::CollectionMut as _;
use justact_core::message::MessageSet as _;
use justact_core::policy::Policy as _;
use justact_core::world::{Interface as _, MessagePool as _};
use justact_policy::datalog::Policy;
use log::{debug, info};

use crate::interface::Interface;
use crate::message::datalog::MessageSet;
use crate::pool::MessagePool;


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
    pool:      MessagePool,
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
        Self { pool: MessagePool::new(), interface, agents: Vec::new() }
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
        Self { pool: MessagePool::new(), interface, agents: Vec::with_capacity(capacity) }
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
        Self { pool: MessagePool::new(), interface, agents }
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

    /// Returns a reference to the internal [`MessagePool`].
    #[inline]
    pub fn pool(&self) -> &MessagePool { &self.pool }

    /// Returns a mutable reference to the internal [`MessagePool`].
    #[inline]
    pub fn pool_mut(&mut self) -> &mut MessagePool { &mut self.pool }

    /// Returns a reference to the internal [`Interface`].
    #[inline]
    pub fn interface(&self) -> &Interface { &self.interface }

    /// Returns a mutable reference to the internal [`Interface`].
    #[inline]
    pub fn interface_mut(&mut self) -> &mut Interface { &mut self.interface }
}
impl<A> Simulation<A>
where
    A: RationalAgent<MessagePool = MessagePool, Interface = Interface>,
{
    /// Polls all the agents in the simulation once.
    ///
    /// # Returns
    /// True if at least one agent is alive, or false otherwise.
    ///
    /// # Errors
    /// This function errors if any of the agents fails to communicate with the end-user or other agents.
    pub fn poll(&mut self) -> Result<bool, Error<A::Error>> {
        let mut i: usize = 0;
        info!("Starting new agent iteration");
        self.agents = self
            .agents
            .drain(..)
            .filter_map(|mut a| {
                i += 1;
                debug!("Polling agent {}...", i - 1);
                match a.poll(&mut self.pool, &mut self.interface) {
                    Ok(AgentPoll::Alive) => Some(Ok(a)),
                    Ok(AgentPoll::Dead) => None,
                    Err(err) => Some(Err(Error::AgentPoll { i: i - 1, err })),
                }
            })
            .collect::<Result<Vec<A>, Error<A::Error>>>()?;
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

            // Check if any actions are unjustified
            if !self.pool.all_actions().is_empty() {
                let mut actions_ok: bool = true;
                for act in self.pool.all_actions() {
                    // Throw the action's message sets in one pile
                    let mut set: MessageSet = MessageSet::empty();
                    set.join(act.basis.as_borrow());
                    set.join(act.justification.as_borrow());
                    set.add(act.enactment.as_borrow());

                    // Get the policy out of it
                    let policy: Policy = set.extract();

                    // Ensure it's OK
                    if let Err(int) = policy.check_validity() {
                        self.interface.error("<system>", format!("Invalid action found\n{act}{int}")).unwrap();
                        actions_ok = false;
                    }
                }
                if actions_ok {
                    self.interface.log("<system>", "All actions are valid").unwrap();
                }
            }

            // Stop if no agents are alive
            if !reiterate {
                return Ok(());
            }
        }
    }
}
