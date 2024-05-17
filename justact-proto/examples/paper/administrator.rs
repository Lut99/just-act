//  ADMINISTRATOR.rs
//    by Lut99
//
//  Created:
//    17 May 2024, 14:23:42
//  Last edited:
//    17 May 2024, 18:25:30
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the administrator-agent from the paper.\
//

use std::convert::Infallible;

use datalog::ast::{datalog, Reserializable, Spec};
use justact_core::auxillary::Identifiable;
use justact_core::{Actions, Agent, AgentPoll, Agreements, Map, RationalAgent, Statements, Times};
use justact_prototype::local::Target;
use justact_prototype::wire::{Action, Message};


/***** LIBRARY *****/
/// The administrator agent, holding all the power.
#[derive(Debug)]
pub struct Administrator;
impl Identifiable for Administrator {
    type Id = &'static str;

    #[inline]
    fn id(&self) -> &Self::Id { &"administrator" }
}
impl Agent for Administrator {}
impl RationalAgent for Administrator {
    type Enactment = Action;
    type Action = Action;
    type Statement = Message;
    type Message = Message;
    type Target = Target;
    type Error = Infallible;

    fn poll<G, L>(&mut self, _global: &mut G, local: &mut L) -> Result<AgentPoll, Self::Error>
    where
        G: Agreements + Times,
        L: Actions<Enactment = Self::Enactment, Action = Self::Action, Target = Self::Target>
            + Statements<Statement = Self::Statement, Message = Self::Message, Target = Self::Target>,
        Self::Error: From<<L as Actions>::Error> + From<<L as Statements>::Error>,
    {
        // The administrator emits 's2' after the agreement has een emitted
        if <L as Map<Self::Message>>::contains(local, &"s1") {
            // Define the policy to emit
            let spec: Spec = datalog! {
                ctl_authorises(administrator, amy, x_rays).
            };
            let msg: Message = Message { id: "s2", author: "administrator", data: spec.reserialize().to_string().into_bytes() };

            // Emit it
            local.state(Target::All, msg).unwrap();

            // The admin is done for this example
            return Ok(AgentPoll::Dead);
        }

        // That's it, this agent is done for the day
        Ok(AgentPoll::Alive)
    }
}
