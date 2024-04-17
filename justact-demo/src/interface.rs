//  INTERFACE.rs
//    by Lut99
//
//  Created:
//    16 Apr 2024, 10:58:56
//  Last edited:
//    17 Apr 2024, 16:33:27
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements a [`justact::Interface`] that allows agents to
//!   communicate with the simulation environment's end user.
//

use std::collections::HashMap;
use std::fmt::Display;

use console::{style, Style};
use justact_core::collection::Collection as _;
use justact_core::message::{Action, Message, MessageSet};
use justact_core::world as justact;


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
}

impl justact::Interface for Interface {
    type Error = std::convert::Infallible;

    fn log(&mut self, id: &str, msg: impl Display) -> Result<(), Self::Error> {
        // Retrieve some style for this agent
        let style: &Style = match self.entities.get(id) {
            Some(style) => style,
            None => panic!("Unregistered entity '{id}'"),
        };

        // Write for that agent
        println!("{}", style.apply_to(format!("[INFO][{id}] {msg}")));
        Ok(())
    }

    fn log_emit<M>(&mut self, id: &str, msg: &M) -> Result<(), Self::Error>
    where
        M: Display + Message,
        M::Author: Display,
        M::Id: Display,
    {
        // Retrieve some style for this agent
        let astyle: &Style = match self.entities.get(id) {
            Some(style) => style,
            None => panic!("Unregistered entity '{id}'"),
        };

        // Write the main log-line
        println!("{}{}{} Emitted message '{}'", style("[INFO] [").bold(), astyle.apply_to(id), style("]").bold(), msg.id());

        // Generate a serialized message
        let smsg: String = msg.to_string().replace('\n', "\n         ");
        let smsg: &str = smsg.trim_end();

        // Write the message
        println!(" └> Message '{}' {{", style(msg.id()).bold());
        println!("        {smsg}");
        println!("    }}");
        println!();

        // Done
        Ok(())
    }

    fn log_enact<'a, A>(&mut self, id: &str, act: &'a A) -> Result<(), Self::Error>
    where
        A: Display + Action,
        <A::Message as Message>::Author: Display,
        <A::Message as Message>::Id: Display,
        <A::MessageSet as MessageSet>::Policy<'a>: Display,
    {
        // Retrieve some style for this agent
        let astyle: &Style = match self.entities.get(id) {
            Some(style) => style,
            None => panic!("Unregistered entity '{id}'"),
        };

        // Retrieve the message IDs for the justication
        let mut just_ids: String = String::new();
        for msg in act.justification().iter() {
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
            act.basis().id(),
            just_ids,
            act.enactment().id()
        );

        // Generate serialized messages
        let sbasis: String = A::MessageSet::from(act.basis()).extract().to_string().replace('\n', "\n         ");
        let sbasis: &str = sbasis.trim_end();

        let sjust: String = act.justification().extract().to_string().replace('\n', "\n         ");
        let sjust: &str = sjust.trim_end();

        let senact: String = A::MessageSet::from(act.enactment()).extract().to_string().replace('\n', "\n         ");
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
        Ok(())
    }

    fn error(&mut self, id: &str, msg: impl std::fmt::Display) -> Result<(), Self::Error> {
        // Write for that agent
        println!("{}", Style::new().bold().red().apply_to(format!("[ERRR][{id}] {msg}")));
        Ok(())
    }
}
