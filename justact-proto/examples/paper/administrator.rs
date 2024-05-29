//  ADMINISTRATOR.rs
//    by Lut99
//
//  Created:
//    17 May 2024, 14:23:42
//  Last edited:
//    27 May 2024, 17:51:10
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the administrator-agent from the paper.
//

use std::convert::Infallible;

use datalog::ast::{datalog, Reserializable, Spec};
use justact_core::agents::{Agent, AgentPoll, RationalAgent};
use justact_core::agreements::Agreements;
use justact_core::auxillary::Identifiable;
use justact_core::statements::Statements;
use justact_core::times::Times;
use justact_prototype::statements::{Message, Target};


/***** LIBRARY *****/
/// The administrator agent, holding all the power.
#[derive(Debug)]
pub struct Administrator;
impl Identifiable for Administrator {
    type Id = str;

    #[inline]
    fn id(&self) -> &Self::Id { "administrator" }
}
impl Agent for Administrator {}
impl RationalAgent for Administrator {
    type Message = Message;
    type Target = Target;
    type Error = Infallible;

    fn poll(
        &mut self,
        agrmnts: impl Agreements<Message = Self::Message>,
        _times: impl Times,
        mut stmts: impl Statements<Message = Self::Message, Target = Self::Target>,
    ) -> Result<AgentPoll, Self::Error> {
        // The administrator emits 's2' after the agreement has een emitted
        if agrmnts.agreed().contains("s1") {
            // Define the policy to emit
            let spec: Spec = datalog! {
                ctl_authorises(administrator, amy, x_rays).
            };
            let msg: Message =
                Message { id: "s2".into(), author: "administrator".into(), payload: spec.reserialize().to_string().into_bytes() };

            // Emit it
            stmts.state(Target::All, msg);

            // The admin is done for this example
            return Ok(AgentPoll::Dead);
        }

        // That's it, this agent is done for the day
        Ok(AgentPoll::Alive)
    }
}
