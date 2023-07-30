//love sonechka-zvezdochka
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::Instruction, native_token::LAMPORTS_PER_SOL, system_program,
};
use anchor_lang::InstructionData;
use clockwork_sdk::state::{Thread, ThreadAccount};
use gpl_session::{session_auth_or, Session, SessionError, SessionToken};

declare_id!("CHPyHid6CQzErEYrsuinBRsjPdsUZdUzgKMCc6VZ9Tjf");

#[error_code]
pub enum GameErrorCode {
    #[msg("Wrong Authority")]
    WrongAuthority,
}

pub const PLAYER_SEED: &[u8] = b"player";
pub const RUN_SEED: &[u8] = b"run";
pub const THREAD_AUTHORITY_SEED: &[u8] = b"thread_authority";

pub const COOLDOWN_BY_TYPE: [u8; 7] = [5, 4, 6, 4, 6, 8, 10];
pub const HEALTH_BY_TYPE: [u8; 7] = [5, 7, 10, 5, 5, 10, 20];
pub const ATTACK_BY_TYPE: [u8; 7] = [2, 1, 3, 1, 1, 2, 5];
pub const CARD_COST_BY_TYPE: [u8; 3] = [1, 2, 3];

// 0 - increase max health
// 1 - increase attack damage
// 2 - attack faster

#[program]
pub mod extracto_program {
    use super::*;

    pub fn init_player(ctx: Context<InitPlayer>, name: String) -> Result<()> {
        let player = &ctx.accounts.player;
        let player_data = &mut ctx.accounts.player_data;
        let run = &mut ctx.accounts.run;

        player_data.authority = player.key();
        player_data.name = name;
        player_data.runs_finished = 0;
        player_data.is_in_run = false;
        player_data.best_score = 0;

        run.authority = player.key();
        run.score = 0;

        Ok(())
    }

    pub fn start_new_run(ctx: Context<StartNewRun>, thread_id: Vec<u8>) -> Result<()> {
        let player = &ctx.accounts.player;
        let run = &mut ctx.accounts.run;
        let player_data = &mut ctx.accounts.player_data;
        let system_program = &ctx.accounts.system_program;
        let clockwork_program = &ctx.accounts.clockwork_program;
        let thread = &ctx.accounts.thread;
        let thread_authority = &ctx.accounts.thread_authority;

        // 1️⃣ Prepare an instruction to be automated.
        let target_ix = Instruction {
            program_id: ID,
            accounts: crate::accounts::IncrementViaThread {
                run: run.key(),
                thread: thread.key(),
                thread_authority: thread_authority.key(),
            }
            .to_account_metas(Some(true)),
            data: crate::instruction::IncrementViaThread {}.data(),
        };

        // 2️⃣ Define a trigger for the thread (every 10 secs).
        let trigger = clockwork_sdk::state::Trigger::Cron {
            schedule: "*/1 * * * * * *".into(),
            skippable: true,
        };

        // 3️⃣ Create thread via CPI.
        let bump = *ctx.bumps.get("thread_authority").unwrap();
        clockwork_sdk::cpi::thread_create(
            CpiContext::new_with_signer(
                clockwork_program.to_account_info(),
                clockwork_sdk::cpi::ThreadCreate {
                    payer: player.to_account_info(),
                    system_program: system_program.to_account_info(),
                    thread: thread.to_account_info(),
                    authority: thread_authority.to_account_info(),
                },
                &[&[THREAD_AUTHORITY_SEED, player.key().as_ref(), &[bump]]], //this is signer seeds needed by the called program to verify PDA signature
            ),
            100000000,              // amount
            thread_id,              // id
            vec![target_ix.into()], // instructions
            trigger,                // trigger
        )?;

        player_data.is_in_run = true;
        run.score = 0;

        run.slots[0] = Some(CharacterInfo {
            id: 0,
            alignment: 0,
            character_type: 0,
            cooldown: COOLDOWN_BY_TYPE[0],
            cooldown_timer: COOLDOWN_BY_TYPE[0],
            max_health: HEALTH_BY_TYPE[0],
            health: HEALTH_BY_TYPE[0],
            attack_damage: ATTACK_BY_TYPE[0],
            state: 0
        });
        run.slots[1] = Some(CharacterInfo {
            id: 1,
            alignment: 0,
            character_type: 1,
            cooldown: COOLDOWN_BY_TYPE[1],
            cooldown_timer: COOLDOWN_BY_TYPE[1],
            max_health: HEALTH_BY_TYPE[1],
            health: HEALTH_BY_TYPE[1],
            attack_damage: ATTACK_BY_TYPE[1],
            state: 0
        });
        run.slots[2] = Some(CharacterInfo {
            id: 2,
            alignment: 0,
            character_type: 2,
            cooldown: COOLDOWN_BY_TYPE[2],
            cooldown_timer: COOLDOWN_BY_TYPE[2],
            max_health: HEALTH_BY_TYPE[2],
            health: HEALTH_BY_TYPE[2],
            attack_damage: ATTACK_BY_TYPE[2],
            state: 0
        });
        run.slots[3] = Some(CharacterInfo {
            id: 3,
            alignment: 1,
            character_type: 3,
            cooldown: COOLDOWN_BY_TYPE[3],
            cooldown_timer: COOLDOWN_BY_TYPE[3],
            max_health: HEALTH_BY_TYPE[3],
            health: HEALTH_BY_TYPE[3],
            attack_damage: ATTACK_BY_TYPE[3],
            state: 0
        });
        run.slots[4] = Some(CharacterInfo {
            id: 4,
            alignment: 1,
            character_type: 4,
            cooldown: COOLDOWN_BY_TYPE[4],
            cooldown_timer: COOLDOWN_BY_TYPE[4],
            max_health: HEALTH_BY_TYPE[4],
            health: HEALTH_BY_TYPE[4],
            attack_damage: ATTACK_BY_TYPE[4],
            state: 0
        });
        run.slots[5] = Some(CharacterInfo {
            id: 5,
            alignment: 1,
            character_type: 5,
            cooldown: COOLDOWN_BY_TYPE[5],
            cooldown_timer: COOLDOWN_BY_TYPE[5],
            max_health: HEALTH_BY_TYPE[5],
            health: HEALTH_BY_TYPE[5],
            attack_damage: ATTACK_BY_TYPE[5],
            state: 0
        });
        run.slots[6] = Some(CharacterInfo {
            id: 6,
            alignment: 1,
            character_type: 6,
            cooldown: COOLDOWN_BY_TYPE[6],
            cooldown_timer: COOLDOWN_BY_TYPE[6],
            max_health: HEALTH_BY_TYPE[6],
            health: HEALTH_BY_TYPE[6],
            attack_damage: ATTACK_BY_TYPE[6],
            state: 0
        });

        run.last_character_id = 6;

        run.cards[0] = CardInfo {
            id: 0,
            card_type: 0,
        };
        run.cards[1] = CardInfo {
            id: 1,
            card_type: 1,
        };
        run.cards[2] = CardInfo {
            id: 2,
            card_type: 2,
        };

        run.last_card_id = 2;

        Ok(())
    }

    pub fn finish_run(ctx: Context<FinishRun>) -> Result<()> {
        let run = &mut ctx.accounts.run;
        let player_data = &mut ctx.accounts.player_data;
        let clockwork_program = &ctx.accounts.clockwork_program;
        let player = &ctx.accounts.player;
        let thread = &ctx.accounts.thread;
        let thread_authority = &ctx.accounts.thread_authority;

        // Delete thread via CPI.
        let bump = *ctx.bumps.get("thread_authority").unwrap();
        clockwork_sdk::cpi::thread_delete(CpiContext::new_with_signer(
            clockwork_program.to_account_info(),
            clockwork_sdk::cpi::ThreadDelete {
                authority: thread_authority.to_account_info(),
                close_to: player.to_account_info(),
                thread: thread.to_account_info(),
            },
            &[&[THREAD_AUTHORITY_SEED, player.key().as_ref(), &[bump]]],
        ))?;

        player_data.runs_finished = player_data.runs_finished.checked_add(1).unwrap();

        if run.score > player_data.best_score {
            player_data.best_score = run.score;
        }

        player_data.is_in_run = false;
        run.score = 0;
        run.experience = 0;
        run.last_character_id = 0;
        run.last_card_id = 0;

        Ok(())
    }

    pub fn start_thread(ctx: Context<StartThread>, thread_id: Vec<u8>) -> Result<()> {
        // Get accounts.
        let system_program = &ctx.accounts.system_program;
        let clockwork_program = &ctx.accounts.clockwork_program;
        let user = &ctx.accounts.user;
        let thread = &ctx.accounts.thread;
        let thread_authority = &ctx.accounts.thread_authority;
        let run = &mut ctx.accounts.run;

        // 1️⃣ Prepare an instruction to be automated.
        let target_ix = Instruction {
            program_id: ID,
            accounts: crate::accounts::IncrementViaThread {
                run: run.key(),
                thread: thread.key(),
                thread_authority: thread_authority.key(),
            }
            .to_account_metas(Some(true)),
            data: crate::instruction::IncrementViaThread {}.data(),
        };

        // 2️⃣ Define a trigger for the thread (every 10 secs).
        let trigger = clockwork_sdk::state::Trigger::Cron {
            schedule: "*/1 * * * * * *".into(),
            skippable: true,
        };

        // 3️⃣ Create thread via CPI.
        let bump = *ctx.bumps.get("thread_authority").unwrap();
        clockwork_sdk::cpi::thread_create(
            CpiContext::new_with_signer(
                clockwork_program.to_account_info(),
                clockwork_sdk::cpi::ThreadCreate {
                    payer: user.to_account_info(),
                    system_program: system_program.to_account_info(),
                    thread: thread.to_account_info(),
                    authority: thread_authority.to_account_info(),
                },
                &[&[THREAD_AUTHORITY_SEED, user.key().as_ref(), &[bump]]], //this is signer seeds needed by the called program to verify PDA signature
            ),
            LAMPORTS_PER_SOL,       // amount
            thread_id,              // id
            vec![target_ix.into()], // instructions
            trigger,                // trigger
        )?;

        Ok(())
    }

    pub fn pause_thread(ctx: Context<PauseThread>) -> Result<()> {
        // Get accounts.
        let user = &ctx.accounts.user;
        let clockwork_program = &ctx.accounts.clockwork_program;
        let thread = &ctx.accounts.thread;
        let thread_authority = &ctx.accounts.thread_authority;

        // 3️⃣ Pause thread via CPI.
        let bump = *ctx.bumps.get("thread_authority").unwrap();
        clockwork_sdk::cpi::thread_pause(
            CpiContext::new_with_signer(
                clockwork_program.to_account_info(),
                clockwork_sdk::cpi::ThreadPause {
                    thread: thread.to_account_info(),
                    authority: thread_authority.to_account_info(),
                },
                &[&[THREAD_AUTHORITY_SEED, user.key().as_ref(), &[bump]]],
            ), // trigger
        )?;

        Ok(())
    }

    pub fn resume_thread(ctx: Context<ResumeThread>) -> Result<()> {
        // Get accounts.
        let user = &ctx.accounts.user;
        let clockwork_program = &ctx.accounts.clockwork_program;
        let thread = &ctx.accounts.thread;
        let thread_authority = &ctx.accounts.thread_authority;

        // 3️⃣ Pause thread via CPI.
        let bump = *ctx.bumps.get("thread_authority").unwrap();
        clockwork_sdk::cpi::thread_resume(
            CpiContext::new_with_signer(
                clockwork_program.to_account_info(),
                clockwork_sdk::cpi::ThreadResume {
                    thread: thread.to_account_info(),
                    authority: thread_authority.to_account_info(),
                },
                &[&[THREAD_AUTHORITY_SEED, user.key().as_ref(), &[bump]]],
            ), // trigger
        )?;

        Ok(())
    }

    pub fn delete_thread(ctx: Context<DeleteThread>) -> Result<()> {
        // Get accounts
        let clockwork_program = &ctx.accounts.clockwork_program;
        let user = &ctx.accounts.user;
        let thread = &ctx.accounts.thread;
        let thread_authority = &ctx.accounts.thread_authority;

        // Delete thread via CPI.
        let bump = *ctx.bumps.get("thread_authority").unwrap();
        clockwork_sdk::cpi::thread_delete(CpiContext::new_with_signer(
            clockwork_program.to_account_info(),
            clockwork_sdk::cpi::ThreadDelete {
                authority: thread_authority.to_account_info(),
                close_to: user.to_account_info(),
                thread: thread.to_account_info(),
            },
            &[&[THREAD_AUTHORITY_SEED, user.key().as_ref(), &[bump]]],
        ))?;
        Ok(())
    }

    pub fn increment_via_thread(ctx: Context<IncrementViaThread>) -> Result<()> {
        let run = &mut ctx.accounts.run;
        run.score = run.score.checked_add(1).unwrap();

        let mut slots_clone = run.slots.clone();
        let n = run.slots.len();

        //refresh cooldowns
        for i in 0..n {
            let slot = slots_clone[i];

            match slot {
                Some(mut character_info) => {
                    //cooldowns
                    let mut perform_action = false;
                    let mut new_cooldown_timer = character_info.cooldown_timer - 1;
                    if new_cooldown_timer == 0 {
                        new_cooldown_timer = character_info.cooldown;
                        perform_action = true;
                    }
                    else{
                        character_info.state = 0;
                    }
                    character_info.cooldown_timer = new_cooldown_timer;

                    slots_clone[i] = Some(character_info);

                    if perform_action {
                        //perform actions for zombies
                        if character_info.alignment == 1 {
                            if i > 0 {
                                if slots_clone[i - 1].is_none() {
                                    //move left
                                    slots_clone[i] = None;
                                    character_info.state = 2;
                                    slots_clone[i - 1] = Some(character_info);
                                }
                                //if there is somebody to the left
                                else {
                                    let mut attacked_character = slots_clone[i - 1].unwrap();

                                    //if it is a hero
                                    if attacked_character.alignment == 0 {
                                        character_info.state = 1;
                                        slots_clone[i] = Some(character_info);
                                        //attack the hero
                                        if character_info.attack_damage >= attacked_character.health
                                        {
                                            slots_clone[i - 1] = None;
                                        } else {
                                            let new_health = attacked_character.health
                                                - character_info.attack_damage;
                                            attacked_character.health = new_health;
                                            slots_clone[i - 1] = Some(attacked_character);
                                        }
                                    }
                                }
                            }
                        }
                        //perform actions for heroes
                        else if character_info.alignment == 0 {
                            match character_info.character_type {
                                2 => {
                                    if slots_clone[i + 1].is_none() {
                                    }
                                    //if there is somebody to the left
                                    else {
                                        let mut attacked_character = slots_clone[i + 1].unwrap();

                                        //if it is a hero
                                        if attacked_character.alignment == 1 {
                                            //attack the hero
                                            character_info.state = 1;
                                            slots_clone[i] = Some(character_info);
                                            if character_info.attack_damage
                                                >= attacked_character.health
                                            {
                                                slots_clone[i + 1] = None;
                                                let new_experience = run.experience + 1;
                                                run.experience = new_experience;
                                            } else {
                                                let new_health = attacked_character.health
                                                    - character_info.attack_damage;
                                                attacked_character.health = new_health;
                                                slots_clone[i + 1] = Some(attacked_character);
                                            }
                                        }
                                    }
                                }
                                1 => {
                                    for a in 4..7 {
                                        if slots_clone[a].is_none() {
                                            //nobody to shoot
                                        } else {
                                            let mut attacked_character = slots_clone[a].unwrap();

                                            //if it is a hero
                                            if attacked_character.alignment == 1 {
                                                //attack the hero
                                                character_info.state = 1;
                                                slots_clone[i] = Some(character_info);
                                                if character_info.attack_damage
                                                    >= attacked_character.health
                                                {
                                                    slots_clone[a] = None;
                                                    let new_experience = run.experience + 1;
                                                    run.experience = new_experience;
                                                } else {
                                                    let new_health = attacked_character.health
                                                        - character_info.attack_damage;
                                                    attacked_character.health = new_health;
                                                    slots_clone[a] = Some(attacked_character);
                                                }
                                            }
                                        }
                                    }
                                }
                                0 => {
                                    if slots_clone[6].is_none() {
                                    } else {
                                        let mut attacked_character = slots_clone[6].unwrap();

                                        if attacked_character.alignment == 1 {
                                            //attack
                                            character_info.state = 1;
                                            slots_clone[i] = Some(character_info);
                                            if character_info.attack_damage
                                                >= attacked_character.health
                                            {
                                                slots_clone[6] = None;
                                                let new_experience = run.experience + 1;
                                                run.experience = new_experience;
                                            } else {
                                                let new_health = attacked_character.health
                                                    - character_info.attack_damage;
                                                attacked_character.health = new_health;
                                                slots_clone[6] = Some(attacked_character);
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if slots_clone[6].is_none() {
            let slot = Clock::get()?.slot;
            let xorshift_output = xorshift64(slot);
            let random_enemy_type_offset = xorshift_output % (4);
            let random_enemy_type = (random_enemy_type_offset + 3) as u8;

            let new_last_character_id = run.last_character_id + 1;
            run.last_character_id = new_last_character_id;

            let new_character_info = CharacterInfo {
                id: new_last_character_id,
                alignment: 1,
                character_type: random_enemy_type,
                cooldown: COOLDOWN_BY_TYPE[random_enemy_type as usize],
                cooldown_timer: COOLDOWN_BY_TYPE[random_enemy_type as usize],
                max_health: HEALTH_BY_TYPE[random_enemy_type as usize],
                health: HEALTH_BY_TYPE[random_enemy_type as usize],
                attack_damage: ATTACK_BY_TYPE[random_enemy_type as usize],
                state: 0
            };

            slots_clone[6] = Some(new_character_info);
        }

        run.slots = slots_clone;

        Ok(())
    }

    #[session_auth_or(
        ctx.accounts.run.authority.key() == ctx.accounts.user.key(),
        GameErrorCode::WrongAuthority
    )]
    pub fn increment(ctx: Context<Increment>) -> Result<()> {
        let run = &mut ctx.accounts.run;
        run.score = run.score.checked_add(1).unwrap();
        Ok(())
    }

    #[session_auth_or(
        ctx.accounts.run.authority.key() == ctx.accounts.user.key(),
        GameErrorCode::WrongAuthority
    )]
    pub fn upgrade(ctx: Context<Upgrade>, card_slot: u16, character_slot_index: u8) -> Result<()> {
        let run = &mut ctx.accounts.run;

        let card_info = run.cards[card_slot as usize];

        let mut character_info = run.slots[character_slot_index as usize].unwrap();

        match card_info.card_type {
            0 => {
                let new_max_health = character_info.max_health + 10;
                character_info.max_health = new_max_health;
                let new_health = character_info.health + 10;
                character_info.health = new_health;
                run.slots[character_slot_index as usize] = Some(character_info);
            }
            1 => {
                let new_attack_damage = character_info.attack_damage + 1;
                character_info.attack_damage = new_attack_damage;
                run.slots[character_slot_index as usize] = Some(character_info);
            }
            2 => {
                let new_cooldown = character_info.cooldown - 1;
                if new_cooldown > 0 {
                    character_info.cooldown = new_cooldown;
                    run.slots[character_slot_index as usize] = Some(character_info);
                }
            }
            _ => {}
        }

        let new_last_card_id = run.last_card_id + 1;
        run.last_card_id = new_last_card_id;

        if run.experience >= CARD_COST_BY_TYPE[card_info.card_type as usize] as u16 {
            let new_experience =
                run.experience - CARD_COST_BY_TYPE[card_info.card_type as usize] as u16;
            run.experience = new_experience;
        }

        let slot = Clock::get()?.slot;
        let xorshift_output = xorshift64(slot);
        let random_card_type = xorshift_output % (3);

        run.cards[card_slot as usize] = CardInfo {
            id: new_last_card_id,
            card_type: random_card_type as u8,
        };

        // replace card with a new one lastCardIndex

        // if (run.experience >= run.cards.)

        run.score = run.score.checked_add(100).unwrap();

        Ok(())
    }

    pub fn reset(ctx: Context<Reset>) -> Result<()> {
        let run = &mut ctx.accounts.run;
        run.score = 0;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(player_name: String)]
pub struct InitPlayer<'info> {
    #[account(
        init,
        payer = player,
        seeds = [PLAYER_SEED, player.key().as_ref()],
        bump,
        space = 8 + 32 + 4 + player_name.len() + 4 + 8 + 1)]
    pub player_data: Account<'info, PlayerData>,
    #[account(
        init,
        payer = player,
        seeds = [RUN_SEED, player.key().as_ref()],
        bump,
        space = 8 + 32 + 8 + 2 + 77 + 2 + 9 + 2)]
    pub run: Account<'info, RunData>,
    #[account(mut)]
    pub player: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(thread_id: Vec<u8>)]
pub struct StartNewRun<'info> {
    #[account(mut, seeds = [RUN_SEED, player.key().as_ref()], bump)]
    pub run: Account<'info, RunData>,

    #[account(mut, seeds = [PLAYER_SEED, player.key().as_ref()], bump)]
    pub player_data: Account<'info, PlayerData>,

    #[account(mut)]
    pub player: Signer<'info>,

    #[account(address = clockwork_sdk::ID)]
    pub clockwork_program: Program<'info, clockwork_sdk::ThreadProgram>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(mut, address = Thread::pubkey(thread_authority.key(), thread_id))]
    pub thread: SystemAccount<'info>,

    #[account(seeds = [THREAD_AUTHORITY_SEED, player.key().as_ref()], bump)]
    pub thread_authority: SystemAccount<'info>,
}

#[derive(Accounts)]
#[instruction(thread_id: Vec<u8>)]
pub struct StartThread<'info> {
    #[account(mut)]
    pub run: Account<'info, RunData>,

    /// The Clockwork thread program.
    #[account(address = clockwork_sdk::ID)]
    pub clockwork_program: Program<'info, clockwork_sdk::ThreadProgram>,

    /// The signer who will pay to initialize the program.
    /// (not to be confused with the thread executions).
    #[account(mut)]
    pub user: Signer<'info>,

    /// The Solana system program.
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    /// Address to assign to the newly created thread.
    #[account(mut, address = Thread::pubkey(thread_authority.key(), thread_id))]
    pub thread: SystemAccount<'info>,

    /// The pda that will own and manage the thread.
    #[account(seeds = [THREAD_AUTHORITY_SEED, user.key().as_ref()], bump)]
    pub thread_authority: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct FinishRun<'info> {
    #[account(mut, seeds = [RUN_SEED, player.key().as_ref()], bump)]
    pub run: Account<'info, RunData>,

    #[account(mut, seeds = [PLAYER_SEED, player.key().as_ref()], bump)]
    pub player_data: Account<'info, PlayerData>,

    #[account(mut)]
    pub player: Signer<'info>,

    #[account(address = clockwork_sdk::ID)]
    pub clockwork_program: Program<'info, clockwork_sdk::ThreadProgram>,

    #[account(mut, address = thread.pubkey(), constraint = thread.authority.eq(&thread_authority.key()))]
    pub thread: Account<'info, Thread>,

    #[account(seeds = [THREAD_AUTHORITY_SEED, player.key().as_ref()], bump)]
    pub thread_authority: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct DeleteThread<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// The Clockwork thread program.
    #[account(address = clockwork_sdk::ID)]
    pub clockwork_program: Program<'info, clockwork_sdk::ThreadProgram>,

    /// The thread to reset.
    #[account(mut, address = thread.pubkey(), constraint = thread.authority.eq(&thread_authority.key()))]
    pub thread: Account<'info, Thread>,

    /// The pda that owns and manages the thread.
    #[account(seeds = [THREAD_AUTHORITY_SEED, user.key().as_ref()], bump)]
    pub thread_authority: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct PauseThread<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// The Clockwork thread program.
    #[account(address = clockwork_sdk::ID)]
    pub clockwork_program: Program<'info, clockwork_sdk::ThreadProgram>,

    /// The thread to pause.
    #[account(mut, address = thread.pubkey(), constraint = thread.authority.eq(&thread_authority.key()))]
    pub thread: Account<'info, Thread>,

    /// The pda that will own and manage the thread.
    #[account(seeds = [THREAD_AUTHORITY_SEED, user.key().as_ref()], bump)]
    pub thread_authority: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct ResumeThread<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// The Clockwork thread program.
    #[account(address = clockwork_sdk::ID)]
    pub clockwork_program: Program<'info, clockwork_sdk::ThreadProgram>,

    /// The thread to reset.
    #[account(mut, address = thread.pubkey(), constraint = thread.authority.eq(&thread_authority.key()))]
    pub thread: Account<'info, Thread>,

    /// The pda that will own and manage the thread.
    #[account(seeds = [THREAD_AUTHORITY_SEED, user.key().as_ref()], bump)]
    pub thread_authority: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct IncrementViaThread<'info> {
    #[account(mut)]
    pub run: Account<'info, RunData>,

    /// Verify that only this thread can execute the Increment Instruction
    #[account(signer, constraint = thread.authority.eq(&thread_authority.key()))]
    pub thread: Account<'info, Thread>,

    /// The Thread Admin
    /// The authority that was used as a seed to derive the thread address
    /// `thread_authority` should equal `thread.thread_authority`
    #[account(seeds = [THREAD_AUTHORITY_SEED, run.authority.key().as_ref()], bump)]
    pub thread_authority: SystemAccount<'info>,
}

#[derive(Accounts, Session)]
pub struct Increment<'info> {
    #[account(mut, seeds = [RUN_SEED, run.authority.key().as_ref()], bump)]
    pub run: Account<'info, RunData>,

    pub user: Signer<'info>,

    #[session(
        // The ephemeral keypair signing the transaction
        signer = user,
        // The authority of the user account which must have created the session
        authority = run.authority.key()
    )]
    // Session Tokens are passed as optional accounts
    pub session_token: Option<Account<'info, SessionToken>>,
}

#[derive(Accounts, Session)]
#[instruction(card_id: u16, slot_id: u8)]
pub struct Upgrade<'info> {
    #[account(mut, seeds = [RUN_SEED, run.authority.key().as_ref()], bump)]
    pub run: Account<'info, RunData>,

    pub user: Signer<'info>,

    #[session(
        // The ephemeral keypair signing the transaction
        signer = user,
        // The authority of the user account which must have created the session
        authority = run.authority.key()
    )]
    // Session Tokens are passed as optional accounts
    pub session_token: Option<Account<'info, SessionToken>>,
}

#[derive(Accounts)]
pub struct Reset<'info> {
    #[account(mut, seeds = [RUN_SEED, user.key().as_ref()], bump)]
    pub run: Account<'info, RunData>,
    pub user: Signer<'info>,
}

#[account]
pub struct RunData {
    //32
    pub authority: Pubkey,
    //8
    pub score: u64,
    //2
    pub experience: u16,
    //(1 + 10) * 7 = 77
    pub slots: [Option<CharacterInfo>; 7],
    //2
    pub last_character_id: u16,
    //(3) * 3 = 9
    pub cards: [CardInfo; 3],
    //2
    pub last_card_id: u16,
}

#[derive(Default, AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Eq)]
// size: 2 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 = 10
pub struct CharacterInfo {
    pub id: u16,
    pub alignment: u8,
    pub character_type: u8,
    pub cooldown: u8,
    pub cooldown_timer: u8,
    pub max_health: u8,
    pub health: u8,
    pub attack_damage: u8,
    pub state: u8
}

impl CharacterInfo {
    pub fn update_timer(&mut self, new_timer: u8) {
        self.cooldown_timer = new_timer;
    }
}

#[derive(Default, AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Eq)]
//size: 2 + 1
pub struct CardInfo {
    pub id: u16,
    pub card_type: u8,
}

#[account]
pub struct PlayerData {
    pub authority: Pubkey,
    pub name: String,
    pub runs_finished: u32,
    pub best_score: u64,
    pub is_in_run: bool,
}

pub fn xorshift64(seed: u64) -> u64 {
    let mut x = seed;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}
