//  INTERFACE.rs
//    by Lut99
//
//  Created:
//    16 Apr 2024, 10:58:56
//  Last edited:
//    13 May 2024, 15:39:53
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements a [`justact::Interface`] that allows agents to
//!   communicate with the simulation environment's end user.
//

use std::borrow::Cow;
use std::fmt::Display;

use console::{style, Style};
use justact_core::set::{MessageSet as _, Set as _};
use justact_core::wire::{Action as _, Message as _};

#[cfg(feature = "datalog")]
use crate::lang::datalog;


/***** LIBRARY *****/
/// Implements a [`justact::Interface`] that allows agents to communicate with the simulation environment's end user.
#[derive(Clone, Debug)]
pub struct Interface {
    /// The style for this agent's interface.
    style: Style,
}

impl Interface {
    /// Constructor for the Interface.
    ///
    /// # Arguments
    /// - `style`: The [`Style`] to initialize this interface logger for.
    ///
    /// # Returns
    /// A new Interface ready for use in the simulation.
    #[inline]
    pub fn new(style: Style) -> Self { Self { style } }

    /// Logs an arbitrary message to stdout.
    ///
    /// # Arguments
    /// - `id`: The identifier of the agent who is logging.
    /// - `msg`: Some message (retrieved as [`Display`]) to show.
    pub fn log(&self, id: &str, msg: impl Display) {
        println!("{}{}{} {}\n", style("[INFO] [").bold(), self.style.apply_to(id), style("]").bold(), msg);
    }

    /// Logs the statement of a datalog [`Message`] to stdout.
    ///
    /// # Arguments
    /// - `id`: The identifier of the agent who is logging.
    /// - `msg`: Some [`datalog::Message`] to emit.
    #[cfg(feature = "datalog")]
    pub fn log_state_datalog(&self, id: &str, msg: &datalog::Message) {
        // Write the main log-line
        println!("{}{}{} Emitted message '{}'", style("[INFO] [").bold(), self.style.apply_to(id), style("]").bold(), msg.id());

        // Generate a serialized message
        let smsg: String = msg.to_string().replace('\n', "\n        ");
        let smsg: &str = smsg.trim_end();

        // Write the message
        println!(" └> Message '{}' {{", style(msg.id()).bold());
        println!("        {smsg}");
        println!("    }}");
        println!();

        // Done
    }

    /// Logs the enactment of a datalog [`Action`] to stdout.
    ///
    /// # Arguments
    /// - `id`: The identifier of the agent who is logging.
    /// - `act`: Some [`datalog::Action`] to emit.
    #[cfg(feature = "datalog")]
    pub fn log_enact_datalog(&self, id: &str, act: &datalog::Action) {
        // Retrieve the message IDs for the justication
        let mut just_ids: String = String::new();
        for msg in act.justification.iter() {
            if !just_ids.is_empty() {
                just_ids.push_str(", ");
            }
            just_ids.push_str(&msg.id().to_string());
        }

        // Write the main log-line
        println!(
            "{}{}{} Enacted message '{}' using '{}' (basis '{}')",
            style("[INFO] [").bold(),
            self.style.apply_to(id),
            style("]").bold(),
            act.enactment().id(),
            just_ids,
            act.basis().id(),
        );

        // Generate serialized messages
        let sbasis: String = datalog::MessageSet::from(Cow::Borrowed(act.basis())).extract().to_string().replace('\n', "\n |      ");
        let sbasis: &str = &sbasis[..sbasis.len() - if sbasis.ends_with("\n |      ") { 9 } else { 0 }];

        let sjust: String = act.justification.extract().to_string().replace('\n', "\n |      ");
        let sjust: &str = &sjust[..sjust.len() - if sjust.ends_with("\n |      ") { 9 } else { 0 }];

        let senact: String = datalog::MessageSet::from(Cow::Borrowed(act.enactment())).extract().to_string().replace('\n', "\n        ");
        let senact: &str = senact.trim_end();

        // Write the sets
        println!(" ├> Basis '{}' {{", style(act.basis().id()).bold());
        println!(" |      {sbasis}");
        println!(" |  }}");
        println!(" ├> Justification '{}' {{", style(just_ids).bold());
        println!(" |      {sjust}");
        println!(" |  }}");
        println!(" └> Enactment '{}' {{", style(act.enactment().id()).bold());
        println!("        {senact}");
        println!("    }}");
        println!();

        // Done
    }

    /// Logs an error message to stdout.
    ///
    /// # Arguments
    /// - `id`: The identifier of the agent who is logging.
    /// - `msg`: Some message (retrieved as [`Display`]) to show.
    pub fn error(&self, id: &str, msg: impl Display) {
        println!(
            "{}{}{}{}{} {}\n",
            style("[").bold(),
            style("ERROR").bold().red(),
            style("] [").bold(),
            self.style.apply_to(id),
            style("]").bold(),
            msg
        );
    }

    /// Logs a the result of a failed audit to stdout.
    ///
    /// # Arguments
    /// - `id`: The identifier of the agent who is logging.
    /// - `act`: The [`datalog::Action`] that failed the audit.
    /// - `expl`: The [`datalog::Explanation`] of why the audit failed.
    #[cfg(feature = "datalog")]
    pub fn error_audit_datalog(&self, id: &str, act: &datalog::Action<'_>, expl: datalog::Explanation) {
        // Write for that agent
        println!(
            "{}{}{}{}{} Action that enacts '{}' did not succeed audit",
            style("[").bold(),
            style("ERROR").bold().red(),
            style("] [").bold(),
            self.style.apply_to(id),
            style("]").bold(),
            act.enactment().id(),
        );

        // Retrieve the message IDs for the justication
        let mut just_ids: String = String::new();
        for msg in act.justification.iter() {
            if !just_ids.is_empty() {
                just_ids.push_str(", ");
            }
            just_ids.push_str(&msg.id().to_string());
        }

        // Generate serialized messages
        let sbasis: String = datalog::MessageSet::from(Cow::Borrowed(act.basis())).extract().to_string().replace('\n', "\n |      ");
        let sbasis: &str = &sbasis[..sbasis.len() - if sbasis.ends_with("\n |      ") { 9 } else { 0 }];

        let sjust: String = act.justification.extract().to_string().replace('\n', "\n |      ");
        let sjust: &str = &sjust[..sjust.len() - if sjust.ends_with("\n |      ") { 9 } else { 0 }];

        let senact: String = datalog::MessageSet::from(Cow::Borrowed(act.enactment())).extract().to_string().replace('\n', "\n        ");
        let senact: &str = senact.trim_end();

        let sint: String = match expl {
            datalog::Explanation::NotStated { message } => format!("Message '{}' is not stated", style(message).bold()),
            datalog::Explanation::Error { int } => format!("{}", int.to_string().replace('\n', "\n    ")),
        };
        let sint: &str = sint.trim_end();

        // Write the sets
        println!(" ├> Basis '{}' {{", style(act.basis().id()).bold());
        println!(" |      {sbasis}");
        println!(" |  }}");
        println!(" ├> Justification '{}' {{", style(just_ids).bold());
        println!(" |      {sjust}");
        println!(" |  }}");
        println!(" ├> Enactment '{}' {{", style(act.enactment().id()).bold());
        println!(" |      {senact}");
        println!(" |  }}");
        println!(" |");
        println!(" └> {sint}");
        println!();
    }
}
