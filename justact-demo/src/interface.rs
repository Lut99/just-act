//  INTERFACE.rs
//    by Lut99
//
//  Created:
//    16 Apr 2024, 10:58:56
//  Last edited:
//    19 Apr 2024, 13:58:55
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements a [`justact::Interface`] that allows agents to
//!   communicate with the simulation environment's end user.
//

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Display;

use console::{style, Style};
use justact_core::message::{Action as _, Message as _};
use justact_core::set::{MessageSet as _, Set as _};

#[cfg(feature = "datalog")]
use crate::lang::datalog;


/***** LIBRARY *****/
/// Implements a [`justact::Interface`] that allows agents to communicate with the simulation environment's end user.
#[derive(Clone, Debug)]
pub struct Interface {
    /// Maps registered entities to some style for them.
    entities: HashMap<String, Style>,
}

impl Default for Interface {
    #[inline]
    fn default() -> Self { Self::new() }
}
impl Interface {
    /// Constructor for the Interface.
    ///
    /// # Returns
    /// A new Interface ready for use in the simulation.
    #[inline]
    pub fn new() -> Self { Self { entities: HashMap::new() } }

    /// Constructor for the Interface that prepares space for some entities.
    ///
    /// # Arguments
    /// - `capacity`: The minimum number of entities that can register before a re-allocation happens.
    ///
    /// # Returns
    /// A new Interface ready for use in the simulation.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self { Self { entities: HashMap::with_capacity(capacity) } }

    /// Registers a new agent with this Interface.
    ///
    /// # Arguments
    /// - `id`: Some identifier to recognize the agent under later.
    /// - `style`: A [`Style`] describing colouring to apply for that agent.
    #[inline]
    pub fn register(&mut self, id: impl Into<String>, style: impl Into<Style>) { self.entities.insert(id.into(), style.into()); }

    /// Logs an arbitrary message to stdout.
    ///
    /// # Arguments
    /// - `id`: The identifier of the agent who is logging.
    /// - `msg`: Some message (retrieved as [`Display`]) to show.
    pub fn log(&self, id: &str, msg: impl Display) {
        // Retrieve some style for this agent
        let astyle: &Style = match self.entities.get(id) {
            Some(style) => style,
            None => panic!("Unregistered entity '{id}'"),
        };

        // Write for that agent
        println!("{}{}{} {}", style("[INFO] [").bold(), astyle.apply_to(id), style("]").bold(), msg);
    }

    /// Logs the statement of a datalog [`Message`] to stdout.
    ///
    /// # Arguments
    /// - `id`: The identifier of the agent who is logging.
    /// - `msg`: Some [`datalog::Message`] to emit.
    #[cfg(feature = "datalog")]
    pub fn log_state_datalog(&self, id: &str, msg: &datalog::Message) {
        // Retrieve some style for this agent
        let astyle: &Style = match self.entities.get(id) {
            Some(style) => style,
            None => panic!("Unregistered entity '{id}'"),
        };

        // Write the main log-line
        println!("{}{}{} Emitted message '{}'", style("[INFO] [").bold(), astyle.apply_to(id), style("]").bold(), msg.id());

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
        // Retrieve some style for this agent
        let astyle: &Style = match self.entities.get(id) {
            Some(style) => style,
            None => panic!("Unregistered entity '{id}'"),
        };

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
            astyle.apply_to(id),
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
        // Retrieve some style for this agent
        let astyle: &Style = match self.entities.get(id) {
            Some(style) => style,
            None => panic!("Unregistered entity '{id}'"),
        };

        // Write for that agent
        println!("{}{}{}{}{} {}", style("[").bold(), style("ERROR").bold().red(), style("] [").bold(), astyle.apply_to(id), style("]").bold(), msg);
    }

    /// Logs a the result of a failed audit to stdout.
    ///
    /// # Arguments
    /// - `id`: The identifier of the agent who is logging.
    /// - `act`: The [`datalog::Action`] that failed the audit.
    /// - `expl`: The [`datalog::Explanation`] of why the audit failed.
    #[cfg(feature = "datalog")]
    pub fn error_audit_datalog(&self, id: &str, act: &datalog::Action<'_>, expl: datalog::Explanation) {
        // Retrieve some style for this agent
        let astyle: &Style = match self.entities.get(id) {
            Some(style) => style,
            None => panic!("Unregistered entity '{id}'"),
        };

        // Write for that agent
        println!(
            "{}{}{}{}{} Action that enacts '{}' did not succeed audit",
            style("[").bold(),
            style("ERROR").bold().red(),
            style("] [").bold(),
            astyle.apply_to(id),
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
