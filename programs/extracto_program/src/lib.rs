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

#[program]
pub mod extracto_program {
    use super::*;

    pub fn init_player(ctx: Context<InitPlayer>, name: String) -> Result<()> {
        let player = &ctx.accounts.player;
        let player_data = &mut ctx.accounts.player_data;

        player_data.authority = player.key();
        player_data.name = name;
        player_data.runs_finished = 0;

        msg!("PlayerData account for {} initialized", player_data.name);
        Ok(())
    }

    pub fn start_new_run(ctx: Context<StartNewRun>) -> Result<()> {
        let user = &ctx.accounts.user;
        let run = &mut ctx.accounts.run;

        run.authority = user.key();
        run.score = 0;

        msg!("RunData account created");
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
            schedule: "*/10 * * * * * *".into(),
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
        msg!("Previous points: {}", run.score);
        run.score = run.score.checked_add(1).unwrap();
        msg!("Run points incremented. Current points: {}", run.score);
        Ok(())
    }

    #[session_auth_or(
        ctx.accounts.run.authority.key() == ctx.accounts.user.key(),
        GameErrorCode::WrongAuthority
    )]
    pub fn increment(ctx: Context<Increment>) -> Result<()> {
        let run = &mut ctx.accounts.run;
        msg!("Previous run points: {}", run.score);
        run.score = run.score.checked_add(1).unwrap();
        msg!("Run points incremented. Current points: {}", run.score);
        Ok(())
    }

    pub fn reset(ctx: Context<Reset>) -> Result<()> {
        let run = &mut ctx.accounts.run;
        run.score = 0;
        msg!("Run points reset. Current points: {}", run.score);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct StartNewRun<'info> {
    #[account(
        init,
        payer = user,
        seeds = [RUN_SEED, user.key().as_ref()],
        bump,
        space = 8 + 32 + 8)]
    pub run: Account<'info, RunData>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
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

#[derive(Accounts)]
pub struct Reset<'info> {
    #[account(mut, seeds = [RUN_SEED, user.key().as_ref()], bump)]
    pub run: Account<'info, RunData>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(player_name: String)]
pub struct InitPlayer<'info> {
    #[account(
        init,
        payer = player,
        seeds = [PLAYER_SEED, player.key().as_ref()],
        bump,
        space = 8 + 32 + 4 + player_name.len() + 4)]
    pub player_data: Account<'info, PlayerData>,
    #[account(mut)]
    pub player: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct RunData {
    pub authority: Pubkey,
    pub score: u64,
}

#[account]
pub struct PlayerData {
    pub authority: Pubkey,
    pub name: String,
    pub runs_finished: u32,
}

pub fn xorshift64(seed: u64) -> u64 {
    let mut x = seed;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}
