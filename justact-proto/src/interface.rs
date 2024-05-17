//  INTERFACE.rs
//    by Lut99
//
//  Created:
//    16 Apr 2024, 10:58:56
//  Last edited:
//    17 May 2024, 14:17:51
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements a collection of users to format messages nicesly.
//

use std::collections::HashMap;
use std::fmt::Display;

use console::{style, Style};
use justact_core::auxillary::Identifiable as _;
use justact_core::{Action as _, Set as _};

use crate::global::Timestamp;
use crate::wire::{Action, Agreement, AuditExplanation, Message};


/***** LIBRARY *****/
/// Implements a [`justact::Interface`] that allows agents to communicate with the simulation environment's end user.
#[derive(Clone, Debug)]
pub struct Interface {
    /// The mapping of agents to their styles.
    styles: HashMap<&'static str, Style>,
}

impl Interface {
    /// Constructor for the Interface.
    ///
    /// # Returns
    /// A new Interface ready for use in the simulation.
    #[inline]
    pub fn new() -> Self { Self { styles: HashMap::new() } }

    /// Registers the style for a new agent.
    ///
    /// Note that all other functions panic if you call it for an agent that hasn't been registered yet.
    ///
    /// # Arguments
    /// - `id`: The identifier of the agent to register.
    /// - `style`: The style to use when formatting the agent's `id`entifier.
    #[inline]
    pub fn register(&mut self, id: &'static str, style: Style) { self.styles.insert(id, style); }



    /// Logs an arbitrary message to stdout.
    ///
    /// # Arguments
    /// - `id`: The identifier of the agent who is logging.
    /// - `msg`: Some message (retrieved as [`Display`]) to show.
    pub fn log(&self, id: &str, msg: impl Display) {
        println!("{}{}{} {}\n", style("[INFO] [").bold(), self.styles.get(id).unwrap().apply_to(id), style("]").bold(), msg);
    }

    /// Logs the statement of a [`Message`] to stdout.
    ///
    /// # Arguments
    /// - `id`: The identifier of the agent who is logging.
    /// - `msg`: Some [`Message`] to emit.
    pub fn log_state(&self, id: &str, msg: &Message) {
        // Write the main log-line
        println!("{}{}{} Emitted message '{}'", style("[INFO] [").bold(), self.styles.get(id).unwrap().apply_to(id), style("]").bold(), msg.id());

        // Write the message
        println!(" └> {}", msg.display("Message", "    "));
        println!();

        // Done
    }

    /// Logs the enactment of a [`Action`] to stdout.
    ///
    /// # Arguments
    /// - `id`: The identifier of the agent who is logging.
    /// - `act`: Some [`Action`] to emit.
    pub fn log_enact(&self, id: &str, act: &Action) {
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
            self.styles.get(id).unwrap().apply_to(id),
            style("]").bold(),
            act.enacts().id(),
            just_ids,
            act.basis().id(),
        );

        // Write the sets
        println!(" ├> {}", act.basis().display("Basis", " |  "));
        println!(" ├> {}", act.justification().display("Justification", " |  "));
        println!(" └> {}", act.enacts().display("Enacts", "    "));
        println!();

        // Done
    }



    /// Logs that an agent started synchronizing a new time.
    ///
    /// # Arguments
    /// - `id`: The identifier of the agent who is logging.
    /// - `time`: The [`Timestamp`] that will be advanced if synchronized.
    pub fn log_advance_start(&self, id: &str, time: Timestamp) {
        // Write the main log-line
        println!(
            "{}{}{} Initiated synchronization to advance time to {}",
            style("[INFO] [").bold(),
            self.styles.get(id).unwrap().apply_to(id),
            style("]").bold(),
            style(time).bold()
        );
        println!();
    }

    /// Logs that all agents agreed to synchronize time.
    ///
    /// # Arguments
    /// - `time`: The [`Timestamp`] that will be advanced if synchronized.
    pub fn log_advance(&self, time: Timestamp) {
        // Write the main log-line
        println!(
            "{}{}{} Time advanced to {}",
            style("[INFO] [").bold(),
            self.styles.get("<system>").unwrap().apply_to("<system>"),
            style("]").bold(),
            style(time).bold()
        );
        println!();
    }

    /// Logs that an agent started synchronizing a new agreement.
    ///
    /// # Arguments
    /// - `id`: The identifier of the agent who is logging.
    /// - `agrmnt`: The [`Agreement`] that will be added to the pool of agreements when synchronized.
    pub fn log_agree_start(&self, id: &str, agrmnt: &Agreement) {
        // Write the main log-line
        println!(
            "{}{}{} Initiated synchronization to agree on message '{}'",
            style("[INFO] [").bold(),
            self.styles.get(id).unwrap().apply_to(id),
            style("]").bold(),
            style(agrmnt.id()).bold()
        );

        // Write the set
        println!(" └> {}", agrmnt.display("Agreement", "    "));
        println!();
    }

    /// Logs that all agents agreed to synchronize time.
    ///
    /// # Arguments
    /// - `agrmnt`: The [`Agreement`] that will be added to the pool of agreements when synchronized.
    pub fn log_agree(&self, agrmnt: &Agreement) {
        // Write the main log-line
        println!(
            "{}{}{} New agreement '{}' created",
            style("[INFO] [").bold(),
            self.styles.get("<system>").unwrap().apply_to("<system>"),
            style("]").bold(),
            style(agrmnt.id()).bold()
        );

        // Write the set
        println!(" └> {}", agrmnt.display("Agreement", "    "));
        println!();
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
            self.styles.get(id).unwrap().apply_to(id),
            style("]").bold(),
            msg
        );
    }

    /// Logs a the result of a failed audit to stdout.
    ///
    /// # Arguments
    /// - `id`: The identifier of the agent who is logging.
    /// - `expl`: The [`Explanation`] of why the audit failed.
    pub fn error_audit<E1, E2>(&self, id: &str, expl: crate::local::AuditExplanation<E1, E2>) {
        // Write for that agent
        println!(
            "{}{}{}{}{} Action that enacts '{}' did not succeed audit",
            style("[").bold(),
            style("ERROR").bold().red(),
            style("] [").bold(),
            self.styles.get(id).unwrap().apply_to(id),
            style("]").bold(),
            expl.act.enacts().id(),
        );

        // Retrieve the message IDs for the justication
        let mut just_ids: String = String::new();
        for msg in expl.act.justification.iter() {
            if !just_ids.is_empty() {
                just_ids.push_str(", ");
            }
            just_ids.push_str(&msg.id().to_string());
        }

        // Generate serialized explanation
        let sexpl: String = match expl.expl {
            AuditExplanation::Stated { stmt } => format!("Message '{}' is not stated", style(stmt).bold()),
            AuditExplanation::Extract { err: _ } => format!("Cannot extract policy"),
            AuditExplanation::Valid { expl: _ } => format!("Extracted policy is not valid"),
            AuditExplanation::Based { stmt } => format!("Message '{}' is not in the set of agreements", style(stmt).bold()),
            AuditExplanation::Timely { stmt, applies_at, taken_at } => format!(
                "Message '{}' is an agreement valid for time {}, but the action was taken at time {}",
                style(stmt).bold(),
                applies_at,
                taken_at
            ),
        };
        let sexpl: &str = sexpl.trim_end();

        // Write the sets
        println!(" ├> {}", expl.act.basis().display("Basis", " |  "));
        println!(" ├> {}", expl.act.justification().display("Justification", " |  "));
        println!(" ├> {}", expl.act.enacts().display("Enacts", "    "));
        println!(" └> {sexpl}");
        println!();
    }
}
