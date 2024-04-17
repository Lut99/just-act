//  PAPER.rs
//    by Lut99
//
//  Created:
//    16 Apr 2024, 11:00:44
//  Last edited:
//    17 Apr 2024, 11:11:25
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the examples from the paper using $Datalog^\neg$ in the
//!   simple simulation environment.
//!   
//!   Contains the full Step 1-example.
//

#[cfg(not(all(feature = "datalog")))]
compile_error!("Please enable the 'datalog'-feature");

use clap::Parser;
use console::Style;
use error_trace::trace;
use humanlog::{DebugMode, HumanLogger};
use justact_core::agent::{Agent, AgentPoll, RationalAgent};
use justact_core::world::{Interface as _, MessagePool as _};
use justact_demo::interface::Interface;
use justact_demo::message::datalog::{Action, Message};
use justact_demo::pool::{MessagePool, Scope};
use justact_demo::Simulation;
use justact_policy::datalog::ast::{datalog, Spec};
use log::{error, info};


/***** ARGUMENTS *****/
/// Defines arguments for this example.
#[derive(Debug, Parser)]
struct Arguments {
    /// If given, enables INFO- and DEBUG-level logging.
    #[clap(long, global = true)]
    debug: bool,
    /// If given, enables INFO-, DEBUG- and TRACE-level logging. Implies '--debug'.
    #[clap(long, global = true)]
    trace: bool,
}





/***** AGENTS *****/
/// An agent abstracting over the other types.
enum AbstractAgent {
    Administrator(Administrator),
    Amy(Amy),
    Anton(Anton),
    Consortium(Consortium),
}
impl Agent for AbstractAgent {
    type Error = std::convert::Infallible;
    type Identifier<'s> = &'static str;

    #[inline]
    fn id<'s>(&'s self) -> Self::Identifier<'s> {
        match self {
            Self::Administrator(a) => a.id(),
            Self::Amy(a) => a.id(),
            Self::Anton(a) => a.id(),
            Self::Consortium(c) => c.id(),
        }
    }
}
impl RationalAgent for AbstractAgent {
    type Interface = Interface;
    type MessagePool = MessagePool;

    fn poll(&mut self, pool: &mut Self::MessagePool, interface: &mut Self::Interface) -> Result<AgentPoll, Self::Error> {
        match self {
            Self::Administrator(a) => a.poll(pool, interface),
            Self::Amy(a) => a.poll(pool, interface),
            Self::Anton(a) => a.poll(pool, interface),
            Self::Consortium(c) => c.poll(pool, interface),
        }
    }
}
impl From<Administrator> for AbstractAgent {
    #[inline]
    fn from(value: Administrator) -> Self { Self::Administrator(value) }
}
impl From<Amy> for AbstractAgent {
    #[inline]
    fn from(value: Amy) -> Self { Self::Amy(value) }
}
impl From<Anton> for AbstractAgent {
    #[inline]
    fn from(value: Anton) -> Self { Self::Anton(value) }
}
impl From<Consortium> for AbstractAgent {
    #[inline]
    fn from(value: Consortium) -> Self { Self::Consortium(value) }
}

/// The consortium agent, authoring messages.
struct Consortium;
impl Consortium {
    /// Constructor for the Consortium.
    ///
    /// # Arguments
    /// - `interface`: Some [`Interface`] to register ourselves with.
    ///
    /// # Returns
    /// A new Consortium agent.
    fn new(interface: &mut Interface) -> Self {
        // Register this agent before returning ourselves
        interface.register("consortium", Style::new().yellow().bold());
        Self {}
    }
}
impl Agent for Consortium {
    type Error = std::convert::Infallible;
    type Identifier<'s> = &'static str;

    #[inline]
    fn id<'s>(&'s self) -> Self::Identifier<'s> { "consortium" }
}
impl RationalAgent for Consortium {
    type Interface = Interface;
    type MessagePool = MessagePool;

    fn poll(&mut self, pool: &mut Self::MessagePool, interface: &mut Self::Interface) -> Result<AgentPoll, Self::Error> {
        // The consortium emits 's1' at the start of the interaction
        if !pool.all_messages().contains_key("s1") {
            // Define the policy to emit
            let spec: Spec = datalog! { #![crate = "::justact_policy::datalog"]
                owns(administrator, Data) :- ctl_accesses(Accessor, Data).
                error :- ctl_accesses(Accessor, Data), owns(Owner, Data), not ctl_authorises(Owner, Accessor, Data).
            };
            let msg: Message = Message::new("s1", "consortium", spec.into());

            // Log it
            interface.log_emit("consortium", &msg)?;

            // Emit it
            pool.emit(msg, Scope::All)?;
        }

        // That's it, this agent is done for the day
        Ok(AgentPoll::Dead)
    }
}

/// The administrator agent, holding all the power.
struct Administrator;
impl Administrator {
    /// Constructor for the Administrator.
    ///
    /// # Arguments
    /// - `interface`: Some [`Interface`] to register ourselves with.
    ///
    /// # Returns
    /// A new Administrator agent.
    fn new(interface: &mut Interface) -> Self {
        // Register this agent before returning ourselves
        interface.register("administrator", Style::new().blue().bold());
        Self {}
    }
}
impl Agent for Administrator {
    type Error = std::convert::Infallible;
    type Identifier<'s> = &'static str;

    #[inline]
    fn id<'s>(&'s self) -> Self::Identifier<'s> { "administrator" }
}
impl RationalAgent for Administrator {
    type Interface = Interface;
    type MessagePool = MessagePool;

    fn poll(&mut self, pool: &mut Self::MessagePool, interface: &mut Self::Interface) -> Result<AgentPoll, Self::Error> {
        // The administrator emits 's2' after the agreement has een emitted
        if pool.all_messages().contains_key("s1") {
            // Define the policy to emit
            let spec: Spec = datalog! { #![crate = "::justact_policy::datalog"]
                ctl_authorises(administrator, amy, x_rays).
            };
            let msg: Message = Message::new("s2", "administrator", spec.into());

            // Log it
            interface.log_emit("administrator", &msg)?;

            // Emit it
            pool.emit(msg, Scope::All)?;

            // The admin is done for this example
            return Ok(AgentPoll::Dead);
        }

        // That's it, this agent is done for the day
        Ok(AgentPoll::Alive)
    }
}

/// The Amy agent, doing the data access
struct Amy;
impl Amy {
    /// Constructor for the Amy.
    ///
    /// # Arguments
    /// - `interface`: Some [`Interface`] to register ourselves with.
    ///
    /// # Returns
    /// A new Amy agent.
    fn new(interface: &mut Interface) -> Self {
        // Register this agent before returning ourselves
        interface.register("amy", Style::new().green().bold());
        Self {}
    }
}
impl Agent for Amy {
    type Error = std::convert::Infallible;
    type Identifier<'s> = &'static str;

    #[inline]
    fn id<'s>(&'s self) -> Self::Identifier<'s> { "amy" }
}
impl RationalAgent for Amy {
    type Interface = Interface;
    type MessagePool = MessagePool;

    fn poll(&mut self, pool: &mut Self::MessagePool, interface: &mut Self::Interface) -> Result<AgentPoll, Self::Error> {
        // The amy emits 's3' (an enacted action) after she received authorisation from the administrator
        if pool.all_messages().contains_key("s2") {
            // Amy first emits her intended enactment
            {
                // The policy to emit
                let spec: Spec = datalog! { #![crate = "::justact_policy::datalog"]
                    ctl_accesses(amy, x_rays).
                };
                let msg: Message = Message::new("s3", "amy", spec.into());

                // Log it
                interface.log_emit("amy", &msg)?;

                // Emit it
                pool.emit(msg, Scope::All)?;
            }

            // Then, she creates an Action
            {
                // The action to emit
                let act: Action = Action {
                    basis: pool.all_messages().get("s1").unwrap().clone().into(),
                    justification: pool.all_messages().get("s2").unwrap().clone().into(),
                    enactment: pool.all_messages().get("s3").unwrap().clone().into(),
                };

                // Log it
                interface.log("amy", "Amy enacts 's3' using 's1' as basis and 's2' as justification")?;

                // Emit it
                pool.enact(act, Scope::All)?;
            }

            // That's Amy's role
            return Ok(AgentPoll::Dead);
        }

        // That's it, this agent is done for the day
        Ok(AgentPoll::Alive)
    }
}

/// The Anton agent, that wreaks havoc.
struct Anton;
impl Anton {
    /// Constructor for the Anton.
    ///
    /// # Arguments
    /// - `interface`: Some [`Interface`] to register ourselves with.
    ///
    /// # Returns
    /// A new Anton agent.
    fn new(interface: &mut Interface) -> Self {
        // Register this agent before returning ourselves
        interface.register("anton", Style::new().magenta().bold());
        Self {}
    }
}
impl Agent for Anton {
    type Error = std::convert::Infallible;
    type Identifier<'s> = &'static str;

    #[inline]
    fn id<'s>(&'s self) -> Self::Identifier<'s> { "anton" }
}
impl RationalAgent for Anton {
    type Interface = Interface;
    type MessagePool = MessagePool;

    fn poll(&mut self, pool: &mut Self::MessagePool, interface: &mut Self::Interface) -> Result<AgentPoll, Self::Error> {
        // Anton emits some malicious messages at the end
        if pool.all_messages().contains_key("s3") && !pool.all_messages().contains_key("s5") {
            // To illustrate, we also emit an action at the end
            {
                // Define the policy to emit
                let spec: Spec = datalog! { #![crate = "::justact_policy::datalog"]
                    ctl_authorises(administrator, anton, x_rays).
                };
                let msg: Message = Message::new("s4", "anton", spec.into());

                // Log it
                interface.log_emit("anton", &msg)?;

                // Emit it
                pool.emit(msg, Scope::All)?;
            }
            {
                // Define the policy to emit
                let spec: Spec = datalog! { #![crate = "::justact_policy::datalog"]
                    ctl_accesses(anton, x_rays).
                };
                let msg: Message = Message::new("s5", "anton", spec.into());

                // Log it
                interface.log_emit("anton", &msg)?;

                // Emit it
                pool.emit(msg, Scope::All)?;
            }
            {
                // The action to emit
                let act: Action = Action {
                    basis: pool.all_messages().get("s1").unwrap().clone().into(),
                    justification: pool.all_messages().get("s4").unwrap().clone().into(),
                    enactment: pool.all_messages().get("s5").unwrap().clone().into(),
                };

                // Log it
                interface.log("anton", "Anton enacts 's5' using 's1' as basis and 's4' as justification")?;

                // Emit it
                pool.enact(act, Scope::All)?;
            }
        } else if pool.all_messages().contains_key("s5") {
            // Define the policy to emit
            let spec: Spec = datalog! { #![crate = "::justact_policy::datalog"]
                owns(anton, x_rays).
            };
            let msg: Message = Message::new("s6", "anton", spec.into());

            // Log it
            interface.log_emit("anton", &msg)?;
            return Ok(AgentPoll::Dead);
        }

        // Wait until it's Anton's moment to shine
        Ok(AgentPoll::Alive)
    }
}





/***** ENTRYPOINT *****/
fn main() {
    // Read CLI args
    let args = Arguments::parse();

    // Setup logger
    if let Err(err) = HumanLogger::terminal(DebugMode::from_flags(args.trace, args.debug)).init() {
        eprintln!("WARNING: Failed to setup logger: {err} (no logging for this session)");
    }
    info!("{} - v{}", env!("CARGO_BIN_NAME"), env!("CARGO_PKG_VERSION"));

    // Build the Simulation
    let mut sim: Simulation<AbstractAgent> = Simulation::with_capacity(1);
    sim.register_with_interface(Consortium::new);
    sim.register_with_interface(Administrator::new);
    sim.register_with_interface(Amy::new);
    sim.register_with_interface(Anton::new);

    // Run it
    println!();
    if let Err(err) = sim.run() {
        error!("{}", trace!(("Failed to run simulation"), err));
        std::process::exit(1);
    };

    // Done!
    println!();
    println!("Done.");
    println!();
}
