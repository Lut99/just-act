//  INTERFACE.rs
//    by Lut99
//
//  Created:
//    16 Apr 2024, 10:58:56
//  Last edited:
//    16 Apr 2024, 16:25:12
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements a [`justact::Interface`] that allows agents to
//!   communicate with the simulation environment's end user.
//

use std::collections::HashMap;

use console::Style;
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

    fn log(&mut self, id: &str, msg: impl std::fmt::Display) -> Result<(), Self::Error> {
        // Retrieve some style for this agent
        let style: &Style = match self.entities.get(id) {
            Some(style) => style,
            None => panic!("Unregistered entity '{id}'"),
        };

        // Write for that agent
        println!("{}", style.apply_to(format!("[INFO][{id}] {msg}")));
        Ok(())
    }

    fn error(&mut self, id: &str, msg: impl std::fmt::Display) -> Result<(), Self::Error> {
        // Write for that agent
        println!("{}", Style::new().bold().red().apply_to(format!("[ERRR][{id}] {msg}")));
        Ok(())
    }
}
