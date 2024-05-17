//  PAPER.rs
//    by Lut99
//
//  Created:
//    16 Apr 2024, 11:00:44
//  Last edited:
//    17 May 2024, 18:22:05
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the examples from the paper using $Datalog^\neg$ in the
//!   simple simulation environment.
//!   
//!   Contains the full Step 1-example.
//

// Modules
mod paper;

// Imports
use clap::Parser;
use console::Style;
use datalog::ast::Spec;
use error_trace::trace;
use humanlog::{DebugMode, HumanLogger};
use justact_prototype::global::Timestamp;
use justact_prototype::sync::Dictatorship;
use justact_prototype::wire::Agreement;
use justact_prototype::Simulation;
use log::{error, info};

use crate::paper::{AbstractAgent, Administrator};


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





// /// The consortium agent, authoring messages.
// #[derive(Debug)]
// struct Consortium {
//     /// The [`Interface`] with which this agent communicates.
//     interface: Interface,
// }
// impl Consortium {
//     /// Constructor for the Consortium.
//     ///
//     /// # Returns
//     /// A new Consortium agent.
//     #[inline]
//     fn new() -> Self { Self { interface: Interface::new(Style::new().yellow().bold()) } }
// }
// impl Agent for Consortium {
//     type Error = std::convert::Infallible;
//     type Identifier = &'static str;

//     #[inline]
//     fn id(&self) -> Self::Identifier { "consortium" }
// }
// impl RationalAgent for Consortium {
//     type Statements<'s> = StatementsMut<'s>;

//     fn poll(&mut self, stmts: &mut Self::Statements<'_>) -> Result<AgentPoll, Self::Error> {
//         // The consortium emits 's1' at the start of the interaction
//         if !stmts.is_stated("s1") {
//             // Define the policy to emit
//             let spec: Spec = datalog! { #![crate = "::justact_policy::datalog"]
//                 owns(administrator, Data) :- ctl_accesses(Accessor, Data).
//                 error :- ctl_accesses(Accessor, Data), owns(Owner, Data), not ctl_authorises(Owner, Accessor, Data).
//             };
//             let msg: Message = Message::new("s1", "consortium", spec.into());

//             // Log it
//             self.interface.log_state_datalog("consortium", &msg);

//             // Emit it
//             stmts.state(Cow::Owned(msg), Scope::All)?;
//         }

//         // That's it, this agent is done for the day
//         Ok(AgentPoll::Dead)
//     }
// }

// /// The Amy agent, doing the data access.
// #[derive(Debug)]
// struct Amy {
//     /// The [`Interface`] with which this agent communicates.
//     interface: Interface,
// }
// impl Amy {
//     /// Constructor for the Amy.
//     ///
//     /// # Returns
//     /// A new Amy agent.
//     #[inline]
//     fn new() -> Self { Self { interface: Interface::new(Style::new().green().bold()) } }
// }
// impl Agent for Amy {
//     type Error = std::convert::Infallible;
//     type Identifier = &'static str;

//     #[inline]
//     fn id(&self) -> Self::Identifier { "amy" }
// }
// impl RationalAgent for Amy {
//     type Statements<'s> = StatementsMut<'s>;

//     fn poll(&mut self, stmts: &mut Self::Statements<'_>) -> Result<AgentPoll, Self::Error> {
//         // The amy emits 's3' (an enacted action) after she received authorisation from the administrator
//         if stmts.is_stated("s2") {
//             // Amy first emits her intended enactment
//             {
//                 // The policy to emit
//                 let spec: Spec = datalog! { #![crate = "::justact_policy::datalog"]
//                     ctl_accesses(amy, x_rays).
//                 };
//                 let msg: Message = Message::new("s3", "amy", spec.into());

//                 // Log it
//                 self.interface.log_state_datalog("amy", &msg);

//                 // Emit it
//                 stmts.state(Cow::Owned(msg), Scope::All)?;
//             }

//             // Then, she creates an Action
//             {
//                 // The action to emit
//                 let act: Action = Action {
//                     basis: Cow::Owned(stmts.get_stated("s1").unwrap()),
//                     justification: Cow::Owned::<Message>(stmts.get_stated("s2").unwrap()).into(),
//                     enactment: Cow::Owned(stmts.get_stated("s3").unwrap()),
//                 };

//                 // Log it
//                 self.interface.log_enact_datalog("amy", &act);

//                 // Emit it
//                 stmts.enact(act, Scope::All)?;
//             }

//             // That's Amy's role
//             return Ok(AgentPoll::Dead);
//         }

//         // That's it, this agent is done for the day
//         Ok(AgentPoll::Alive)
//     }
// }

// /// The Anton agent, that wreaks havoc.
// #[derive(Debug)]
// struct Anton {
//     /// The [`Interface`] with which this agent communicates.
//     interface: Interface,
// }
// impl Anton {
//     /// Constructor for the Anton.
//     ///
//     /// # Returns
//     /// A new Anton agent.
//     #[inline]
//     fn new() -> Self { Self { interface: Interface::new(Style::new().magenta().bold()) } }
// }
// impl Agent for Anton {
//     type Error = std::convert::Infallible;
//     type Identifier = &'static str;

//     #[inline]
//     fn id(&self) -> Self::Identifier { "anton" }
// }
// impl RationalAgent for Anton {
//     type Statements<'s> = StatementsMut<'s>;

//     fn poll(&mut self, stmts: &mut Self::Statements<'_>) -> Result<AgentPoll, Self::Error> {
//         // Anton emits some malicious messages at the end
//         if stmts.is_stated("s3") && !stmts.is_stated("s5") {
//             // To illustrate, we also emit an action at the end
//             {
//                 // Define the policy to emit
//                 let spec: Spec = datalog! { #![crate = "::justact_policy::datalog"]
//                     ctl_authorises(administrator, anton, x_rays).
//                 };
//                 let msg: Message = Message::new("s4", "anton", spec.into());

//                 // Log it
//                 self.interface.log_state_datalog("anton", &msg);

//                 // Emit it
//                 stmts.state(Cow::Owned(msg), Scope::All)?;
//             }
//             {
//                 // Define the policy to emit
//                 let spec: Spec = datalog! { #![crate = "::justact_policy::datalog"]
//                     ctl_accesses(anton, x_rays).
//                 };
//                 let msg: Message = Message::new("s5", "anton", spec.into());

//                 // Log it
//                 self.interface.log_state_datalog("anton", &msg);

//                 // Emit it
//                 stmts.state(Cow::Owned(msg), Scope::All)?;
//             }
//             {
//                 // The action to emit
//                 let act: Action = Action {
//                     basis: Cow::Owned(stmts.get_stated("s1").unwrap()),
//                     justification: Cow::Owned::<Message>(stmts.get_stated("s4").unwrap()).into(),
//                     enactment: Cow::Owned(stmts.get_stated("s5").unwrap()),
//                 };

//                 // Log it
//                 self.interface.log_enact_datalog("anton", &act);

//                 // Emit it
//                 stmts.enact(act, Scope::All)?;
//             }
//         } else if stmts.is_stated("s5") {
//             // Define the policy to emit
//             let spec: Spec = datalog! { #![crate = "::justact_policy::datalog"]
//                 owns(anton, x_rays).
//             };
//             let msg: Message = Message::new("s6", "anton", spec.into());

//             // Log it
//             self.interface.log_state_datalog("anton", &msg);

//             // Emit it
//             stmts.state(Cow::Owned(msg), Scope::All)?;

//             // That's Anton's work forever
//             return Ok(AgentPoll::Dead);
//         }

//         // Wait until it's Anton's moment to shine
//         Ok(AgentPoll::Alive)
//     }
// }





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
    let mut sim: Simulation<Dictatorship<Agreement>, Dictatorship<Timestamp>, AbstractAgent> =
        Simulation::with_capacity(Dictatorship::new("consortium"), Dictatorship::new("consortium"), 1);
    // sim.register(Consortium, Style::new());
    sim.register(Administrator, Style::new());
    // sim.register(Amy, Style::new());
    // sim.register(Anton, Style::new());

    // Run it
    println!();
    if let Err(err) = sim.run::<Spec>() {
        error!("{}", trace!(("Failed to run simulation"), err));
        std::process::exit(1);
    };

    // Done!
    println!();
    println!("Done.");
    println!();
}
