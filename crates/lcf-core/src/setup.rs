use crate::generated::ldb_gen::Actor;
use crate::types::Parameters;

pub struct Setup;

impl Setup {
    /// Sets up default parameters, level caps, and stats for an Actor.
    pub fn actor(actor: &mut Actor, is_2k3: bool) {
        let max_final_level: usize = if is_2k3 {
            if actor.final_level == -1 || actor.final_level == 0 || actor.final_level == 50 {
                actor.final_level = 99;
            }
            if actor.exp_base == -1 || actor.exp_base == 0 || actor.exp_base == 30 {
                actor.exp_base = 300;
            }
            if actor.exp_inflation == -1 || actor.exp_inflation == 0 || actor.exp_inflation == 30 {
                actor.exp_inflation = 300;
            }
            99
        } else {
            if actor.final_level == -1 || actor.final_level == 0 {
                actor.final_level = 50;
            }
            if actor.exp_base == -1 || actor.exp_base == 0 {
                actor.exp_base = 30;
            }
            if actor.exp_inflation == -1 || actor.exp_inflation == 0 {
                actor.exp_inflation = 30;
            }
            50
        };

        Self::parameters(&mut actor.parameters, max_final_level);
    }


    /// Sets up default parameter stat arrays up to the specified level cap.
    pub fn parameters(params: &mut Parameters, final_level: usize) {
        if params.maxhp.len() < final_level {
            params.maxhp.resize(final_level, 1);
        }
        if params.maxsp.len() < final_level {
            params.maxsp.resize(final_level, 0);
        }
        if params.attack.len() < final_level {
            params.attack.resize(final_level, 1);
        }
        if params.defense.len() < final_level {
            params.defense.resize(final_level, 1);
        }
        if params.spirit.len() < final_level {
            params.spirit.resize(final_level, 1);
        }
        if params.agility.len() < final_level {
            params.agility.resize(final_level, 1);
        }
    }
}
