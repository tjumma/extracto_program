use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::Instruction, native_token::LAMPORTS_PER_SOL, system_program,
};
use anchor_lang::InstructionData;
use clockwork_sdk::state::{Thread, ThreadAccount};

declare_id!("CHPyHid6CQzErEYrsuinBRsjPdsUZdUzgKMCc6VZ9Tjf");

pub const COUNTER_SEED: &[u8] = b"counter";
pub const THREAD_AUTHORITY_SEED: &[u8] = b"thread_authority";

#[program]
pub mod extracto_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let user = &ctx.accounts.user;

        let counter = &mut ctx.accounts.counter;
        counter.authority = user.key();
        counter.count = 0;
        msg!("Counter account created. Current count: {}", counter.count);
        Ok(())
    }

    pub fn start_thread(ctx: Context<StartThread>, thread_id: Vec<u8>) -> Result<()> {
        // Get accounts.
        let system_program = &ctx.accounts.system_program;
        let clockwork_program = &ctx.accounts.clockwork_program;
        let user = &ctx.accounts.user;
        let thread = &ctx.accounts.thread;
        let thread_authority = &ctx.accounts.thread_authority;
        let counter = &mut ctx.accounts.counter;

        // 1️⃣ Prepare an instruction to be automated.
        let target_ix = Instruction {
            program_id: ID,
            accounts: crate::accounts::IncrementViaThread {
                counter: counter.key(),
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
        let counter = &mut ctx.accounts.counter;
        msg!("Previous counter: {}", counter.count);
        counter.count = counter.count.checked_add(1).unwrap();
        msg!("Counter incremented. Current count: {}", counter.count);
        Ok(())
    }

    pub fn increment(ctx: Context<Increment>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        msg!("Previous counter: {}", counter.count);
        counter.count = counter.count.checked_add(1).unwrap();
        msg!("Counter incremented. Current count: {}", counter.count);
        Ok(())
    }

    pub fn reset(ctx: Context<Reset>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.count = 0;
        msg!("Counter reset. Current count: {}", counter.count);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = user,
        seeds = [COUNTER_SEED, user.key().as_ref()],
        bump,
        space = 8 + 8 + 32)]
    pub counter: Account<'info, Counter>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(thread_id: Vec<u8>)]
pub struct StartThread<'info> {
    #[account(mut)]
    pub counter: Account<'info, Counter>,

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
    pub counter: Account<'info, Counter>,

    /// Verify that only this thread can execute the Increment Instruction
    #[account(signer, constraint = thread.authority.eq(&thread_authority.key()))]
    pub thread: Account<'info, Thread>,

    /// The Thread Admin
    /// The authority that was used as a seed to derive the thread address
    /// `thread_authority` should equal `thread.thread_authority`
    #[account(seeds = [THREAD_AUTHORITY_SEED, counter.authority.key().as_ref()], bump)]
    pub thread_authority: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct Increment<'info> {
    #[account(mut, seeds = [COUNTER_SEED, user.key().as_ref()], bump)]
    pub counter: Account<'info, Counter>,

    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct Reset<'info> {
    #[account(mut, seeds = [COUNTER_SEED, user.key().as_ref()], bump)]
    pub counter: Account<'info, Counter>,
    pub user: Signer<'info>,
}

#[account]
pub struct Counter {
    pub authority: Pubkey,
    pub count: u64,
}
