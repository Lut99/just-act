//  PAPER.rs
//    by Lut99
//
//  Created:
//    16 Apr 2024, 11:00:44
//  Last edited:
//    13 May 2024, 14:45:38
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

use std::borrow::Cow;

use clap::Parser;
use console::Style;
use error_trace::trace;
use humanlog::{DebugMode, HumanLogger};
use justact_core::agent::{Agent, AgentPoll, RationalAgent};
use justact_core::local::{Statements as _, Stating};
use justact_demo::interface::Interface;
use justact_demo::lang::datalog::{Action, Message};
use justact_demo::statements::{Scope, StatementsMut};
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
#[derive(Debug)]
enum AbstractAgent {
    Administrator(Administrator),
    Amy(Amy),
    Anton(Anton),
    Consortium(Consortium),
}
impl Agent for AbstractAgent {
    type Error = std::convert::Infallible;
    type Identifier = &'static str;

    #[inline]
    fn id(&self) -> Self::Identifier {
        match self {
            Self::Administrator(a) => a.id(),
            Self::Amy(a) => a.id(),
            Self::Anton(a) => a.id(),
            Self::Consortium(c) => c.id(),
        }
    }
}
impl RationalAgent for AbstractAgent {
    type Statements<'s> = StatementsMut<'s>;

    fn poll(&mut self, pool: &mut Self::Statements<'_>) -> Result<AgentPoll, Self::Error> {
        match self {
            Self::Administrator(a) => a.poll(pool),
            Self::Amy(a) => a.poll(pool),
            Self::Anton(a) => a.poll(pool),
            Self::Consortium(c) => c.poll(pool),
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
#[derive(Debug)]
struct Consortium {
    /// The [`Interface`] with which this agent communicates.
    interface: Interface,
}
impl Consortium {
    /// Constructor for the Consortium.
    ///
    /// # Returns
    /// A new Consortium agent.
    #[inline]
    fn new() -> Self { Self { interface: Interface::new(Style::new().yellow().bold()) } }
}
impl Agent for Consortium {
    type Error = std::convert::Infallible;
    type Identifier = &'static str;

    #[inline]
    fn id(&self) -> Self::Identifier { "consortium" }
}
impl RationalAgent for Consortium {
    type Statements<'s> = StatementsMut<'s>;

    fn poll(&mut self, stmts: &mut Self::Statements<'_>) -> Result<AgentPoll, Self::Error> {
        // The consortium emits 's1' at the start of the interaction
        if !stmts.is_stated("s1") {
            // Define the policy to emit
            let spec: Spec = datalog! { #![crate = "::justact_policy::datalog"]
                owns(administrator, Data) :- ctl_accesses(Accessor, Data).
                error :- ctl_accesses(Accessor, Data), owns(Owner, Data), not ctl_authorises(Owner, Accessor, Data).
            };
            let msg: Message = Message::new("s1", "consortium", spec.into());

            // Log it
            self.interface.log_state_datalog("consortium", &msg);

            // Emit it
            stmts.state(Cow::Owned(msg), Scope::All)?;
        }

        // That's it, this agent is done for the day
        Ok(AgentPoll::Dead)
    }
}

/// The administrator agent, holding all the power.
#[derive(Debug)]
struct Administrator {
    /// The [`Interface`] with which this agent communicates.
    interface: Interface,
}
impl Administrator {
    /// Constructor for the Administrator.
    ///
    /// # Returns
    /// A new Administrator agent.
    #[inline]
    fn new() -> Self { Self { interface: Interface::new(Style::new().blue().bold()) } }
}
impl Agent for Administrator {
    type Error = std::convert::Infallible;
    type Identifier = &'static str;

    #[inline]
    fn id(&self) -> Self::Identifier { "administrator" }
}
impl RationalAgent for Administrator {
    type Statements<'s> = StatementsMut<'s>;

    fn poll(&mut self, stmts: &mut Self::Statements<'_>) -> Result<AgentPoll, Self::Error> {
        // The administrator emits 's2' after the agreement has een emitted
        if stmts.is_stated("s1") {
            // Define the policy to emit
            let spec: Spec = datalog! { #![crate = "::justact_policy::datalog"]
                ctl_authorises(administrator, amy, x_rays).
            };
            let msg: Message = Message::new("s2", "administrator", spec.into());

            // Log it
            self.interface.log_state_datalog("administrator", &msg);

            // Emit it
            stmts.state(Cow::Owned(msg), Scope::All)?;

            // The admin is done for this example
            return Ok(AgentPoll::Dead);
        }

        // That's it, this agent is done for the day
        Ok(AgentPoll::Alive)
    }
}

/// The Amy agent, doing the data access.
#[derive(Debug)]
struct Amy {
    /// The [`Interface`] with which this agent communicates.
    interface: Interface,
}
impl Amy {
    /// Constructor for the Amy.
    ///
    /// # Returns
    /// A new Amy agent.
    #[inline]
    fn new() -> Self { Self { interface: Interface::new(Style::new().green().bold()) } }
}
impl Agent for Amy {
    type Error = std::convert::Infallible;
    type Identifier = &'static str;

    #[inline]
    fn id(&self) -> Self::Identifier { "amy" }
}
impl RationalAgent for Amy {
    type Statements<'s> = StatementsMut<'s>;

    fn poll(&mut self, stmts: &mut Self::Statements<'_>) -> Result<AgentPoll, Self::Error> {
        // The amy emits 's3' (an enacted action) after she received authorisation from the administrator
        if stmts.is_stated("s2") {
            // Amy first emits her intended enactment
            {
                // The policy to emit
                let spec: Spec = datalog! { #![crate = "::justact_policy::datalog"]
                    ctl_accesses(amy, x_rays).
                };
                let msg: Message = Message::new("s3", "amy", spec.into());

                // Log it
                self.interface.log_state_datalog("amy", &msg);

                // Emit it
                stmts.state(Cow::Owned(msg), Scope::All)?;
            }

            // Then, she creates an Action
            {
                // The action to emit
                let act: Action = Action {
                    basis: Cow::Owned(stmts.get_stated("s1").unwrap()),
                    justification: Cow::Owned::<Message>(stmts.get_stated("s2").unwrap()).into(),
                    enactment: Cow::Owned(stmts.get_stated("s3").unwrap()),
                };

                // Log it
                self.interface.log_enact_datalog("amy", &act);

                // Emit it
                stmts.enact(act, Scope::All)?;
            }

            // That's Amy's role
            return Ok(AgentPoll::Dead);
        }

        // That's it, this agent is done for the day
        Ok(AgentPoll::Alive)
    }
}

/// The Anton agent, that wreaks havoc.
#[derive(Debug)]
struct Anton {
    /// The [`Interface`] with which this agent communicates.
    interface: Interface,
}
impl Anton {
    /// Constructor for the Anton.
    ///
    /// # Returns
    /// A new Anton agent.
    #[inline]
    fn new() -> Self { Self { interface: Interface::new(Style::new().magenta().bold()) } }
}
impl Agent for Anton {
    type Error = std::convert::Infallible;
    type Identifier = &'static str;

    #[inline]
    fn id(&self) -> Self::Identifier { "anton" }
}
impl RationalAgent for Anton {
    type Statements<'s> = StatementsMut<'s>;

    fn poll(&mut self, stmts: &mut Self::Statements<'_>) -> Result<AgentPoll, Self::Error> {
        // Anton emits some malicious messages at the end
        if stmts.is_stated("s3") && !stmts.is_stated("s5") {
            // To illustrate, we also emit an action at the end
            {
                // Define the policy to emit
                let spec: Spec = datalog! { #![crate = "::justact_policy::datalog"]
                    ctl_authorises(administrator, anton, x_rays).
                };
                let msg: Message = Message::new("s4", "anton", spec.into());

                // Log it
                self.interface.log_state_datalog("anton", &msg);

                // Emit it
                stmts.state(Cow::Owned(msg), Scope::All)?;
            }
            {
                // Define the policy to emit
                let spec: Spec = datalog! { #![crate = "::justact_policy::datalog"]
                    ctl_accesses(anton, x_rays).
                };
                let msg: Message = Message::new("s5", "anton", spec.into());

                // Log it
                self.interface.log_state_datalog("anton", &msg);

                // Emit it
                stmts.state(Cow::Owned(msg), Scope::All)?;
            }
            {
                // The action to emit
                let act: Action = Action {
                    basis: Cow::Owned(stmts.get_stated("s1").unwrap()),
                    justification: Cow::Owned::<Message>(stmts.get_stated("s4").unwrap()).into(),
                    enactment: Cow::Owned(stmts.get_stated("s5").unwrap()),
                };

                // Log it
                self.interface.log_enact_datalog("anton", &act);

                // Emit it
                stmts.enact(act, Scope::All)?;
            }
        } else if stmts.is_stated("s5") {
            // Define the policy to emit
            let spec: Spec = datalog! { #![crate = "::justact_policy::datalog"]
                owns(anton, x_rays).
            };
            let msg: Message = Message::new("s6", "anton", spec.into());

            // Log it
            self.interface.log_state_datalog("anton", &msg);

            // Emit it
            stmts.state(Cow::Owned(msg), Scope::All)?;

            // That's Anton's work forever
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
    sim.register(Consortium::new());
    sim.register(Administrator::new());
    sim.register(Amy::new());
    sim.register(Anton::new());

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
