use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions::{
    load_current_index_checked, load_instruction_at_checked,
};

// Placeholder program id for local development; replace before deployment.
declare_id!("3C9uAwPX6Bbx2CybpPa1pWA7kFh5xsQLCPA4hbvkHDWE");

const MAX_REACTION_KIND_LEN: usize = 32;
const ABS_MAX_COMMENT_LEN: usize = 2_000;
const MAX_REACTION_KIND_SEED_LEN: usize = MAX_REACTION_KIND_LEN;
const DELETE_TOMBSTONE: &str = "[deleted]";

// ===== Program Instructions (`#[program]`) =====
#[program]
pub mod project_comments {
    use super::*;

    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        super_admin: Pubkey,
        eligibility_mode: u8,
        max_comment_len: u16,
        max_reactions_per_user: u16,
    ) -> Result<()> {
        validate_eligibility_mode(eligibility_mode)?;
        validate_max_comment_len(max_comment_len)?;

        let config = &mut ctx.accounts.config;
        let now = Clock::get()?.unix_timestamp;

        config.super_admin = super_admin;
        config.paused = false;
        config.eligibility_mode = eligibility_mode;
        config.max_comment_len = max_comment_len;
        config.max_reactions_per_user = max_reactions_per_user;
        config.created_at = now;
        config.updated_at = now;
        config.bump = ctx.bumps.config;

        emit!(ConfigInitialized {
            super_admin,
            paused: false,
            eligibility_mode,
            max_comment_len,
            max_reactions_per_user,
        });

        Ok(())
    }

    pub fn update_config(
        ctx: Context<UpdateConfig>,
        paused: bool,
        eligibility_mode: u8,
        max_comment_len: u16,
        max_reactions_per_user: u16,
    ) -> Result<()> {
        validate_eligibility_mode(eligibility_mode)?;
        validate_max_comment_len(max_comment_len)?;

        let config = &mut ctx.accounts.config;
        require_keys_eq!(
            ctx.accounts.admin.key(),
            config.super_admin,
            ProjectCommentsError::Unauthorized
        );

        config.paused = paused;
        config.eligibility_mode = eligibility_mode;
        config.max_comment_len = max_comment_len;
        config.max_reactions_per_user = max_reactions_per_user;
        config.updated_at = Clock::get()?.unix_timestamp;

        emit!(ConfigUpdated {
            super_admin: config.super_admin,
            paused,
            eligibility_mode,
            max_comment_len,
            max_reactions_per_user,
        });

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_thread_for_post(
        ctx: Context<CreateThreadForPost>,
        source_program_id: Pubkey,
        post_id: u64,
    ) -> Result<()> {
        ensure_signer(&ctx.accounts.authority.to_account_info())?;
        ensure_not_paused_or_admin(&ctx.accounts.config, &ctx.accounts.authority.key())?;
        verify_instructions_sysvar(&ctx.accounts.instructions_sysvar)?;

        // Security: owner check before deserialization.
        verify_account_owner(&ctx.accounts.post_meta, &source_program_id)?;
        let post_meta = read_post_meta_standard(&ctx.accounts.post_meta)?;
        validate_post_meta(
            &post_meta,
            source_program_id,
            ctx.accounts.post_account.key(),
            post_id,
        )?;

        let thread = &mut ctx.accounts.thread;
        assert_canonical_pda(
            &thread.key(),
            &[
                b"thread",
                source_program_id.as_ref(),
                ctx.accounts.post_account.key().as_ref(),
            ],
            ctx.bumps.thread,
        )?;

        let now = Clock::get()?.unix_timestamp;
        thread.source_program_id = source_program_id;
        thread.post_account = ctx.accounts.post_account.key();
        thread.post_author = post_meta.post_author;
        thread.post_id = post_id;
        thread.comments_count = 0;
        thread.is_locked = false;
        thread.created_at = now;
        thread.updated_at = now;
        thread.bump = ctx.bumps.thread;

        emit!(ThreadCreated {
            thread: thread.key(),
            source_program_id,
            post_account: thread.post_account,
            post_author: thread.post_author,
            post_id,
        });

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_comment(
        ctx: Context<CreateComment>,
        source_program_id: Pubkey,
        body: String,
    ) -> Result<()> {
        ensure_signer(&ctx.accounts.author.to_account_info())?;
        ensure_not_paused_or_admin(&ctx.accounts.config, &ctx.accounts.author.key())?;
        ensure_thread_writable(&ctx.accounts.thread)?;
        verify_instructions_sysvar(&ctx.accounts.instructions_sysvar)?;

        validate_body(&body, usize::from(ctx.accounts.config.max_comment_len))?;

        verify_account_owner(&ctx.accounts.user_meta, &source_program_id)?;
        let user_meta = read_user_meta_standard(&ctx.accounts.user_meta)?;
        validate_user_eligibility(
            &user_meta,
            source_program_id,
            ctx.accounts.author.key(),
            ctx.accounts.config.eligibility_mode,
        )?;

        let thread_key = ctx.accounts.thread.key();
        let next_comment_id = ctx.accounts.thread.comments_count;

        let thread = &mut ctx.accounts.thread;
        thread.comments_count = thread
            .comments_count
            .checked_add(1)
            .ok_or(ProjectCommentsError::MathOverflow)?;
        thread.updated_at = Clock::get()?.unix_timestamp;

        let comment = &mut ctx.accounts.comment;
        assert_canonical_pda(
            &comment.key(),
            &[
                b"comment",
                thread_key.as_ref(),
                &next_comment_id.to_le_bytes(),
            ],
            ctx.bumps.comment,
        )?;

        let now = Clock::get()?.unix_timestamp;
        comment.thread = thread_key;
        comment.comment_id = next_comment_id;
        comment.author = ctx.accounts.author.key();
        comment.parent_comment = None;
        comment.root_comment = comment.key();
        comment.body = body;
        comment.depth = 0;
        comment.replies_count = 0;
        comment.reactions_total = 0;
        comment.edited_at = None;
        comment.deleted = false;
        comment.created_at = now;
        comment.updated_at = now;
        comment.bump = ctx.bumps.comment;

        emit!(CommentCreated {
            thread: thread_key,
            comment: comment.key(),
            comment_id: next_comment_id,
            author: comment.author,
            parent_comment: None,
            root_comment: comment.root_comment,
            depth: 0,
        });

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reply_comment(
        ctx: Context<ReplyComment>,
        source_program_id: Pubkey,
        body: String,
    ) -> Result<()> {
        ensure_signer(&ctx.accounts.author.to_account_info())?;
        ensure_not_paused_or_admin(&ctx.accounts.config, &ctx.accounts.author.key())?;
        ensure_thread_writable(&ctx.accounts.thread)?;
        verify_instructions_sysvar(&ctx.accounts.instructions_sysvar)?;

        validate_body(&body, usize::from(ctx.accounts.config.max_comment_len))?;

        verify_account_owner(&ctx.accounts.user_meta, &source_program_id)?;
        let user_meta = read_user_meta_standard(&ctx.accounts.user_meta)?;
        validate_user_eligibility(
            &user_meta,
            source_program_id,
            ctx.accounts.author.key(),
            ctx.accounts.config.eligibility_mode,
        )?;

        let thread_key = ctx.accounts.thread.key();
        let parent = &mut ctx.accounts.parent_comment;
        require_keys_eq!(
            parent.thread,
            thread_key,
            ProjectCommentsError::ParentThreadMismatch
        );

        let next_comment_id = ctx.accounts.thread.comments_count;
        let next_depth = parent
            .depth
            .checked_add(1)
            .ok_or(ProjectCommentsError::MathOverflow)?;

        parent.replies_count = parent
            .replies_count
            .checked_add(1)
            .ok_or(ProjectCommentsError::MathOverflow)?;
        parent.updated_at = Clock::get()?.unix_timestamp;

        let thread = &mut ctx.accounts.thread;
        thread.comments_count = thread
            .comments_count
            .checked_add(1)
            .ok_or(ProjectCommentsError::MathOverflow)?;
        thread.updated_at = Clock::get()?.unix_timestamp;

        let comment = &mut ctx.accounts.comment;
        assert_canonical_pda(
            &comment.key(),
            &[
                b"comment",
                thread_key.as_ref(),
                &next_comment_id.to_le_bytes(),
            ],
            ctx.bumps.comment,
        )?;

        let now = Clock::get()?.unix_timestamp;
        comment.thread = thread_key;
        comment.comment_id = next_comment_id;
        comment.author = ctx.accounts.author.key();
        comment.parent_comment = Some(parent.key());
        comment.root_comment = parent.root_comment;
        comment.body = body;
        comment.depth = next_depth;
        comment.replies_count = 0;
        comment.reactions_total = 0;
        comment.edited_at = None;
        comment.deleted = false;
        comment.created_at = now;
        comment.updated_at = now;
        comment.bump = ctx.bumps.comment;

        emit!(CommentCreated {
            thread: thread_key,
            comment: comment.key(),
            comment_id: next_comment_id,
            author: comment.author,
            parent_comment: Some(parent.key()),
            root_comment: comment.root_comment,
            depth: next_depth,
        });

        Ok(())
    }

    pub fn edit_comment(ctx: Context<EditComment>, body: String) -> Result<()> {
        ensure_signer(&ctx.accounts.author.to_account_info())?;
        ensure_not_paused_or_admin(&ctx.accounts.config, &ctx.accounts.author.key())?;
        verify_instructions_sysvar(&ctx.accounts.instructions_sysvar)?;
        validate_body(&body, usize::from(ctx.accounts.config.max_comment_len))?;

        let comment = &mut ctx.accounts.comment;
        require!(!comment.deleted, ProjectCommentsError::CommentDeleted);
        require_keys_eq!(
            comment.author,
            ctx.accounts.author.key(),
            ProjectCommentsError::Unauthorized
        );

        comment.body = body;
        let now = Clock::get()?.unix_timestamp;
        comment.edited_at = Some(now);
        comment.updated_at = now;

        emit!(CommentEdited {
            thread: comment.thread,
            comment: comment.key(),
            editor: ctx.accounts.author.key(),
        });

        Ok(())
    }

    pub fn delete_comment(ctx: Context<DeleteComment>) -> Result<()> {
        ensure_signer(&ctx.accounts.actor.to_account_info())?;
        ensure_not_paused_or_admin(&ctx.accounts.config, &ctx.accounts.actor.key())?;
        verify_instructions_sysvar(&ctx.accounts.instructions_sysvar)?;

        let comment = &mut ctx.accounts.comment;
        let is_author = comment.author == ctx.accounts.actor.key();
        let is_post_author = ctx.accounts.thread.post_author == ctx.accounts.actor.key();
        let is_admin = ctx.accounts.config.super_admin == ctx.accounts.actor.key();

        require!(
            is_author || is_post_author || is_admin,
            ProjectCommentsError::Unauthorized
        );

        comment.deleted = true;
        comment.body = DELETE_TOMBSTONE.to_string();
        comment.updated_at = Clock::get()?.unix_timestamp;

        emit!(CommentDeleted {
            thread: comment.thread,
            comment: comment.key(),
            actor: ctx.accounts.actor.key(),
        });

        Ok(())
    }

    pub fn set_thread_lock(ctx: Context<SetThreadLock>, is_locked: bool) -> Result<()> {
        ensure_signer(&ctx.accounts.actor.to_account_info())?;
        ensure_not_paused_or_admin(&ctx.accounts.config, &ctx.accounts.actor.key())?;
        verify_instructions_sysvar(&ctx.accounts.instructions_sysvar)?;

        let thread = &mut ctx.accounts.thread;
        let is_post_author = thread.post_author == ctx.accounts.actor.key();
        let is_admin = ctx.accounts.config.super_admin == ctx.accounts.actor.key();
        require!(is_post_author || is_admin, ProjectCommentsError::Unauthorized);

        thread.is_locked = is_locked;
        thread.updated_at = Clock::get()?.unix_timestamp;

        emit!(ThreadLockChanged {
            thread: thread.key(),
            actor: ctx.accounts.actor.key(),
            is_locked,
        });

        Ok(())
    }

    pub fn add_reaction(
        ctx: Context<AddReaction>,
        reaction_kind: String,
        _expected_cpi_program: Pubkey,
    ) -> Result<()> {
        ensure_signer(&ctx.accounts.user.to_account_info())?;
        ensure_not_paused_or_admin(&ctx.accounts.config, &ctx.accounts.user.key())?;
        ensure_thread_writable(&ctx.accounts.thread)?;
        verify_instructions_sysvar(&ctx.accounts.instructions_sysvar)?;

        validate_reaction_kind(&reaction_kind)?;

        // Security hardening placeholder: this instruction currently has no CPI paths,
        // but the validator is provided to avoid unchecked future CPI additions.
        verify_pinned_program_id(
            &ctx.accounts.pinned_cpi_program,
            &ctx.accounts.pinned_cpi_program.key(),
        )?;

        let reaction = &mut ctx.accounts.reaction;
        assert_canonical_pda(
            &reaction.key(),
            &[
                b"reaction",
                ctx.accounts.comment.key().as_ref(),
                ctx.accounts.user.key().as_ref(),
                reaction_kind.as_bytes(),
            ],
            ctx.bumps.reaction,
        )?;

        let now = Clock::get()?.unix_timestamp;
        reaction.comment = ctx.accounts.comment.key();
        reaction.user = ctx.accounts.user.key();
        reaction.reaction_kind = reaction_kind.clone();
        reaction.created_at = now;
        reaction.bump = ctx.bumps.reaction;

        let comment = &mut ctx.accounts.comment;
        comment.reactions_total = comment
            .reactions_total
            .checked_add(1)
            .ok_or(ProjectCommentsError::MathOverflow)?;
        comment.updated_at = now;

        emit!(ReactionAdded {
            thread: ctx.accounts.thread.key(),
            comment: comment.key(),
            reaction: reaction.key(),
            user: ctx.accounts.user.key(),
            reaction_kind,
        });

        Ok(())
    }

    pub fn remove_reaction(ctx: Context<RemoveReaction>) -> Result<()> {
        ensure_signer(&ctx.accounts.user.to_account_info())?;
        ensure_not_paused_or_admin(&ctx.accounts.config, &ctx.accounts.user.key())?;
        verify_instructions_sysvar(&ctx.accounts.instructions_sysvar)?;

        require_keys_eq!(
            ctx.accounts.reaction.user,
            ctx.accounts.user.key(),
            ProjectCommentsError::Unauthorized
        );

        let comment = &mut ctx.accounts.comment;
        comment.reactions_total = comment
            .reactions_total
            .checked_sub(1)
            .ok_or(ProjectCommentsError::MathUnderflow)?;
        comment.updated_at = Clock::get()?.unix_timestamp;

        emit!(ReactionRemoved {
            thread: ctx.accounts.thread.key(),
            comment: comment.key(),
            reaction: ctx.accounts.reaction.key(),
            actor: ctx.accounts.user.key(),
            reaction_kind: ctx.accounts.reaction.reaction_kind.clone(),
        });

        Ok(())
    }

    pub fn admin_remove_reaction(ctx: Context<AdminRemoveReaction>) -> Result<()> {
        ensure_signer(&ctx.accounts.actor.to_account_info())?;
        verify_instructions_sysvar(&ctx.accounts.instructions_sysvar)?;

        let actor = ctx.accounts.actor.key();
        let is_admin = actor == ctx.accounts.config.super_admin;
        let is_post_author = actor == ctx.accounts.thread.post_author;
        require!(is_admin || is_post_author, ProjectCommentsError::Unauthorized);

        let comment = &mut ctx.accounts.comment;
        comment.reactions_total = comment
            .reactions_total
            .checked_sub(1)
            .ok_or(ProjectCommentsError::MathUnderflow)?;
        comment.updated_at = Clock::get()?.unix_timestamp;

        emit!(ReactionRemoved {
            thread: ctx.accounts.thread.key(),
            comment: comment.key(),
            reaction: ctx.accounts.reaction.key(),
            actor,
            reaction_kind: ctx.accounts.reaction.reaction_kind.clone(),
        });

        Ok(())
    }
}

// ===== Account Contexts (`#[derive(Accounts)]`) =====
#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(init, payer = payer, space = Config::LEN, seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(mut, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub admin: Signer<'info>,
}

// ===== Account Contexts With Instruction Args (`#[instruction(...)]`) =====
#[derive(Accounts)]
#[instruction(source_program_id: Pubkey)]
pub struct CreateThreadForPost<'info> {
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    #[account(
        init,
        payer = authority,
        space = Thread::LEN,
        seeds = [b"thread", source_program_id.as_ref(), post_account.key().as_ref()],
        bump
    )]
    pub thread: Account<'info, Thread>,
    /// CHECK: validated via explicit owner check + deserialization.
    pub post_meta: UncheckedAccount<'info>,
    /// CHECK: identity pubkey used for deterministic PDA derivation.
    pub post_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: verified against canonical instructions sysvar.
    pub instructions_sysvar: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(_source_program_id: Pubkey)]
pub struct CreateComment<'info> {
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        seeds = [b"thread", thread.source_program_id.as_ref(), thread.post_account.as_ref()],
        bump = thread.bump
    )]
    pub thread: Account<'info, Thread>,
    #[account(
        init,
        payer = author,
        space = Comment::LEN,
        seeds = [b"comment", thread.key().as_ref(), &thread.comments_count.to_le_bytes()],
        bump
    )]
    pub comment: Account<'info, Comment>,
    /// CHECK: validated via explicit owner check + deserialization.
    pub user_meta: UncheckedAccount<'info>,
    #[account(mut)]
    pub author: Signer<'info>,
    /// CHECK: verified against canonical instructions sysvar.
    pub instructions_sysvar: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(_source_program_id: Pubkey)]
pub struct ReplyComment<'info> {
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        seeds = [b"thread", thread.source_program_id.as_ref(), thread.post_account.as_ref()],
        bump = thread.bump
    )]
    pub thread: Account<'info, Thread>,
    #[account(
        mut,
        seeds = [b"comment", thread.key().as_ref(), &parent_comment.comment_id.to_le_bytes()],
        bump = parent_comment.bump
    )]
    pub parent_comment: Account<'info, Comment>,
    #[account(
        init,
        payer = author,
        space = Comment::LEN,
        seeds = [b"comment", thread.key().as_ref(), &thread.comments_count.to_le_bytes()],
        bump
    )]
    pub comment: Account<'info, Comment>,
    /// CHECK: validated via explicit owner check + deserialization.
    pub user_meta: UncheckedAccount<'info>,
    #[account(mut)]
    pub author: Signer<'info>,
    /// CHECK: verified against canonical instructions sysvar.
    pub instructions_sysvar: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EditComment<'info> {
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        seeds = [b"comment", comment.thread.as_ref(), &comment.comment_id.to_le_bytes()],
        bump = comment.bump
    )]
    pub comment: Account<'info, Comment>,
    pub author: Signer<'info>,
    /// CHECK: verified against canonical instructions sysvar.
    pub instructions_sysvar: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct DeleteComment<'info> {
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    #[account(
        seeds = [b"thread", thread.source_program_id.as_ref(), thread.post_account.as_ref()],
        bump = thread.bump
    )]
    pub thread: Account<'info, Thread>,
    #[account(
        mut,
        seeds = [b"comment", comment.thread.as_ref(), &comment.comment_id.to_le_bytes()],
        bump = comment.bump
    )]
    pub comment: Account<'info, Comment>,
    pub actor: Signer<'info>,
    /// CHECK: verified against canonical instructions sysvar.
    pub instructions_sysvar: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct SetThreadLock<'info> {
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        seeds = [b"thread", thread.source_program_id.as_ref(), thread.post_account.as_ref()],
        bump = thread.bump
    )]
    pub thread: Account<'info, Thread>,
    pub actor: Signer<'info>,
    /// CHECK: verified against canonical instructions sysvar.
    pub instructions_sysvar: UncheckedAccount<'info>,
}

#[derive(Accounts)]
#[instruction(reaction_kind: String)]
pub struct AddReaction<'info> {
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    #[account(
        seeds = [b"thread", thread.source_program_id.as_ref(), thread.post_account.as_ref()],
        bump = thread.bump
    )]
    pub thread: Account<'info, Thread>,
    #[account(
        mut,
        seeds = [b"comment", comment.thread.as_ref(), &comment.comment_id.to_le_bytes()],
        bump = comment.bump,
        constraint = comment.thread == thread.key() @ ProjectCommentsError::ParentThreadMismatch
    )]
    pub comment: Account<'info, Comment>,
    #[account(
        init,
        payer = user,
        space = Reaction::LEN,
        seeds = [
            b"reaction",
            comment.key().as_ref(),
            user.key().as_ref(),
            reaction_kind.as_bytes(),
        ],
        bump
    )]
    pub reaction: Account<'info, Reaction>,
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: explicit CPI program pinning guard input.
    pub pinned_cpi_program: UncheckedAccount<'info>,
    /// CHECK: verified against canonical instructions sysvar.
    pub instructions_sysvar: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RemoveReaction<'info> {
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    #[account(
        seeds = [b"thread", thread.source_program_id.as_ref(), thread.post_account.as_ref()],
        bump = thread.bump
    )]
    pub thread: Account<'info, Thread>,
    #[account(
        mut,
        seeds = [b"comment", comment.thread.as_ref(), &comment.comment_id.to_le_bytes()],
        bump = comment.bump,
        constraint = comment.thread == thread.key() @ ProjectCommentsError::ParentThreadMismatch
    )]
    pub comment: Account<'info, Comment>,
    #[account(
        mut,
        close = user,
        seeds = [
            b"reaction",
            reaction.comment.as_ref(),
            reaction.user.as_ref(),
            reaction.reaction_kind.as_bytes(),
        ],
        bump = reaction.bump,
        constraint = reaction.comment == comment.key() @ ProjectCommentsError::ReactionNotFound
    )]
    pub reaction: Account<'info, Reaction>,
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: verified against canonical instructions sysvar.
    pub instructions_sysvar: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct AdminRemoveReaction<'info> {
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    #[account(
        seeds = [b"thread", thread.source_program_id.as_ref(), thread.post_account.as_ref()],
        bump = thread.bump
    )]
    pub thread: Account<'info, Thread>,
    #[account(
        mut,
        seeds = [b"comment", comment.thread.as_ref(), &comment.comment_id.to_le_bytes()],
        bump = comment.bump,
        constraint = comment.thread == thread.key() @ ProjectCommentsError::ParentThreadMismatch
    )]
    pub comment: Account<'info, Comment>,
    #[account(
        mut,
        close = recipient,
        seeds = [
            b"reaction",
            reaction.comment.as_ref(),
            reaction.user.as_ref(),
            reaction.reaction_kind.as_bytes(),
        ],
        bump = reaction.bump,
        constraint = reaction.comment == comment.key() @ ProjectCommentsError::ReactionNotFound
    )]
    pub reaction: Account<'info, Reaction>,
    pub actor: Signer<'info>,
    #[account(mut)]
    /// CHECK: lamport recipient for closing reaction account.
    pub recipient: UncheckedAccount<'info>,
    /// CHECK: verified against canonical instructions sysvar.
    pub instructions_sysvar: UncheckedAccount<'info>,
}

// ===== On-chain Accounts (`#[account]`) =====
// Nota: los `impl` se mantienen junto a cada `#[account]` porque definen
// metadatos del layout (por ejemplo `LEN`) usados en `space = ...` al inicializar PDAs.
// Tenerlos aquí evita desalineaciones entre el schema de la cuenta y su tamaño reservado.
#[account]
pub struct Config {
    pub super_admin: Pubkey,
    pub paused: bool,
    pub eligibility_mode: u8,
    pub max_comment_len: u16,
    pub max_reactions_per_user: u16,
    pub created_at: i64,
    pub updated_at: i64,
    pub bump: u8,
}

// `impl` de `Config`: centraliza constantes/funciones asociadas al tipo de cuenta.
// Aquí se usa para definir `LEN`, que reserva el espacio correcto al inicializar la PDA.
impl Config {
    // Tamaño total serializado de `Config` (8 bytes discriminator + campos).
    pub const LEN: usize = 8 + 32 + 1 + 1 + 2 + 2 + 8 + 8 + 1;
}

#[account]
pub struct Thread {
    pub source_program_id: Pubkey,
    pub post_account: Pubkey,
    pub post_author: Pubkey,
    pub post_id: u64,
    pub comments_count: u64,
    pub is_locked: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub bump: u8,
}

// `impl` de `Thread`: agrupa metadatos del layout de la cuenta.
// `Thread::LEN` se usa en `space = ...` para crear la cuenta con tamaño exacto.
impl Thread {
    // Tamaño total serializado de `Thread` para `space = Thread::LEN`.
    pub const LEN: usize = 8 + 32 + 32 + 32 + 8 + 8 + 1 + 8 + 8 + 1;
}

#[account]
pub struct Comment {
    pub thread: Pubkey,
    pub comment_id: u64,
    pub author: Pubkey,
    pub parent_comment: Option<Pubkey>,
    pub root_comment: Pubkey,
    pub body: String,
    pub depth: u32,
    pub replies_count: u64,
    pub reactions_total: u64,
    pub edited_at: Option<i64>,
    pub deleted: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub bump: u8,
}

// `impl` de `Comment`: mantiene la definición del tamaño máximo serializado.
// Es clave para reservar bytes suficientes cuando el cuerpo del comentario varía.
impl Comment {
    // Tamaño máximo serializado de `Comment` considerando límites de strings.
    pub const LEN: usize = 8
        + 32
        + 8
        + 32
        + (1 + 32)
        + 32
        + (4 + ABS_MAX_COMMENT_LEN)
        + 4
        + 8
        + 8
        + (1 + 8)
        + 1
        + 8
        + 8
        + 1;
}

#[account]
pub struct Reaction {
    pub comment: Pubkey,
    pub user: Pubkey,
    pub reaction_kind: String,
    pub created_at: i64,
    pub bump: u8,
}

// `impl` de `Reaction`: define constantes asociadas a esta cuenta.
// Se usa para calcular `Reaction::LEN` y evitar desbordes/falta de espacio al crearla.
impl Reaction {
    // Tamaño total serializado de `Reaction` para reservar espacio exacto.
    pub const LEN: usize = 8 + 32 + 32 + (4 + MAX_REACTION_KIND_SEED_LEN) + 8 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct PostMetaStandard {
    pub source_program_id: Pubkey,
    pub post_account: Pubkey,
    pub post_author: Pubkey,
    pub post_id: u64,
    pub is_public: bool,
    pub is_archived: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct UserMetaStandard {
    pub source_program_id: Pubkey,
    pub user: Pubkey,
    pub is_registered: bool,
    pub is_eligible_to_comment: bool,
}

// ===== Events (`#[event]`) =====
#[event]
pub struct ConfigInitialized {
    pub super_admin: Pubkey,
    pub paused: bool,
    pub eligibility_mode: u8,
    pub max_comment_len: u16,
    pub max_reactions_per_user: u16,
}

#[event]
pub struct ConfigUpdated {
    pub super_admin: Pubkey,
    pub paused: bool,
    pub eligibility_mode: u8,
    pub max_comment_len: u16,
    pub max_reactions_per_user: u16,
}

#[event]
pub struct ThreadCreated {
    pub thread: Pubkey,
    pub source_program_id: Pubkey,
    pub post_account: Pubkey,
    pub post_author: Pubkey,
    pub post_id: u64,
}

#[event]
pub struct ThreadLockChanged {
    pub thread: Pubkey,
    pub actor: Pubkey,
    pub is_locked: bool,
}

#[event]
pub struct CommentCreated {
    pub thread: Pubkey,
    pub comment: Pubkey,
    pub comment_id: u64,
    pub author: Pubkey,
    pub parent_comment: Option<Pubkey>,
    pub root_comment: Pubkey,
    pub depth: u32,
}

#[event]
pub struct CommentEdited {
    pub thread: Pubkey,
    pub comment: Pubkey,
    pub editor: Pubkey,
}

#[event]
pub struct CommentDeleted {
    pub thread: Pubkey,
    pub comment: Pubkey,
    pub actor: Pubkey,
}

#[event]
pub struct ReactionAdded {
    pub thread: Pubkey,
    pub comment: Pubkey,
    pub reaction: Pubkey,
    pub user: Pubkey,
    pub reaction_kind: String,
}

#[event]
pub struct ReactionRemoved {
    pub thread: Pubkey,
    pub comment: Pubkey,
    pub reaction: Pubkey,
    pub actor: Pubkey,
    pub reaction_kind: String,
}

// ===== Errors (`#[error_code]`) =====
#[error_code]
pub enum ProjectCommentsError {
    #[msg("Unauthorized operation")]
    Unauthorized,
    #[msg("Program is paused for non-admin writes")]
    ProgramPaused,
    #[msg("Thread is locked")]
    ThreadLocked,
    #[msg("Invalid source metadata")]
    InvalidSourceMetadata,
    #[msg("User not eligible to comment")]
    NotEligibleToComment,
    #[msg("Parent comment does not belong to target thread")]
    ParentThreadMismatch,
    #[msg("Comment was deleted")]
    CommentDeleted,
    #[msg("Comment body cannot be empty")]
    BodyEmpty,
    #[msg("Comment body is too long")]
    BodyTooLong,
    #[msg("Invalid reaction kind length")]
    InvalidReactionKind,
    #[msg("Reaction already exists")]
    AlreadyReacted,
    #[msg("Reaction not found")]
    ReactionNotFound,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Math underflow")]
    MathUnderflow,
    #[msg("Invalid eligibility mode")]
    InvalidEligibilityMode,
    #[msg("Invalid max comment length")]
    InvalidMaxCommentLen,
    #[msg("Invalid CPI program")]
    InvalidCpiProgram,
    #[msg("Invalid PDA")]
    InvalidPda,
    #[msg("Invalid account owner")]
    InvalidAccountOwner,
    #[msg("Missing required signer")]
    MissingRequiredSigner,
    #[msg("Invalid sysvar account")]
    InvalidSysvarAccount,
    #[msg("Invalid instruction introspection")]
    InvalidInstructionIntrospection,
}

fn validate_eligibility_mode(mode: u8) -> Result<()> {
    require!(mode <= 2, ProjectCommentsError::InvalidEligibilityMode);
    Ok(())
}

fn validate_max_comment_len(max_comment_len: u16) -> Result<()> {
    require!(max_comment_len > 0, ProjectCommentsError::InvalidMaxCommentLen);
    require!(
        usize::from(max_comment_len) <= ABS_MAX_COMMENT_LEN,
        ProjectCommentsError::InvalidMaxCommentLen
    );
    Ok(())
}

fn validate_body(body: &str, max_comment_len: usize) -> Result<()> {
    if body.is_empty() {
        return err!(ProjectCommentsError::BodyEmpty);
    }

    if body.len() > max_comment_len {
        return err!(ProjectCommentsError::BodyTooLong);
    }

    Ok(())
}

fn validate_reaction_kind(kind: &str) -> Result<()> {
    let len = kind.len();
    require!(
        (1..=MAX_REACTION_KIND_LEN).contains(&len),
        ProjectCommentsError::InvalidReactionKind
    );
    Ok(())
}

fn ensure_thread_writable(thread: &Account<Thread>) -> Result<()> {
    require!(!thread.is_locked, ProjectCommentsError::ThreadLocked);
    Ok(())
}

fn ensure_not_paused_or_admin(config: &Account<Config>, actor: &Pubkey) -> Result<()> {
    if config.paused && *actor != config.super_admin {
        return err!(ProjectCommentsError::ProgramPaused);
    }
    Ok(())
}

fn ensure_signer(account: &AccountInfo) -> Result<()> {
    if !account.is_signer {
        return err!(ProjectCommentsError::MissingRequiredSigner);
    }
    Ok(())
}

fn verify_account_owner(account: &UncheckedAccount, expected_owner: &Pubkey) -> Result<()> {
    if account.owner != expected_owner {
        return err!(ProjectCommentsError::InvalidAccountOwner);
    }
    Ok(())
}

fn read_post_meta_standard(account: &UncheckedAccount) -> Result<PostMetaStandard> {
    decode_external_metadata::<PostMetaStandard>(account)
}

fn read_user_meta_standard(account: &UncheckedAccount) -> Result<UserMetaStandard> {
    decode_external_metadata::<UserMetaStandard>(account)
}

fn decode_external_metadata<T>(account: &UncheckedAccount) -> Result<T>
where
    T: AnchorDeserialize,
{
    let data = account.try_borrow_data()?;

    // Accept both raw-serialized structs and Anchor-account-prefixed bytes.
    let parsed = T::try_from_slice(&data)
        .or_else(|_| {
            if data.len() <= 8 {
                return Err(anchor_lang::error::Error::from(
                    ProjectCommentsError::InvalidSourceMetadata,
                ));
            }
            T::try_from_slice(&data[8..]).map_err(|_| {
                anchor_lang::error::Error::from(ProjectCommentsError::InvalidSourceMetadata)
            })
        })
        .map_err(|_| anchor_lang::error::Error::from(ProjectCommentsError::InvalidSourceMetadata))?;

    Ok(parsed)
}

fn validate_post_meta(
    post_meta: &PostMetaStandard,
    source_program_id: Pubkey,
    post_account: Pubkey,
    post_id: u64,
) -> Result<()> {
    require_keys_eq!(
        post_meta.source_program_id,
        source_program_id,
        ProjectCommentsError::InvalidSourceMetadata
    );
    require_keys_eq!(
        post_meta.post_account,
        post_account,
        ProjectCommentsError::InvalidSourceMetadata
    );
    require!(
        post_meta.post_id == post_id,
        ProjectCommentsError::InvalidSourceMetadata
    );
    require!(post_meta.is_public, ProjectCommentsError::InvalidSourceMetadata);
    require!(
        !post_meta.is_archived,
        ProjectCommentsError::InvalidSourceMetadata
    );
    Ok(())
}

fn validate_user_eligibility(
    user_meta: &UserMetaStandard,
    source_program_id: Pubkey,
    user: Pubkey,
    eligibility_mode: u8,
) -> Result<()> {
    require_keys_eq!(
        user_meta.source_program_id,
        source_program_id,
        ProjectCommentsError::InvalidSourceMetadata
    );
    require_keys_eq!(
        user_meta.user,
        user,
        ProjectCommentsError::InvalidSourceMetadata
    );

    let allowed = match eligibility_mode {
        0 => user_meta.is_registered && user_meta.is_eligible_to_comment,
        1 => user_meta.is_registered,
        2 => true,
        _ => return err!(ProjectCommentsError::InvalidEligibilityMode),
    };

    require!(allowed, ProjectCommentsError::NotEligibleToComment);
    Ok(())
}

fn assert_canonical_pda(account_key: &Pubkey, seeds: &[&[u8]], bump: u8) -> Result<()> {
    let (canonical_pda, canonical_bump) = Pubkey::find_program_address(seeds, &crate::ID);
    if *account_key != canonical_pda || bump != canonical_bump {
        return err!(ProjectCommentsError::InvalidPda);
    }
    Ok(())
}

fn verify_pinned_program_id(
    account: &UncheckedAccount,
    expected_program_id: &Pubkey,
) -> Result<()> {
    if account.key() != *expected_program_id {
        return err!(ProjectCommentsError::InvalidCpiProgram);
    }
    if !account.executable {
        return err!(ProjectCommentsError::InvalidCpiProgram);
    }
    Ok(())
}

fn verify_instructions_sysvar(account: &UncheckedAccount) -> Result<()> {
    let instructions_id = anchor_lang::solana_program::sysvar::instructions::ID;
    if account.key() != instructions_id {
        return err!(ProjectCommentsError::InvalidSysvarAccount);
    }

    let account_info = account.to_account_info();
    let current_index = load_current_index_checked(&account_info)
        .map_err(|_| error!(ProjectCommentsError::InvalidInstructionIntrospection))?;

    load_instruction_at_checked(usize::from(current_index), &account_info)
        .map_err(|_| error!(ProjectCommentsError::InvalidInstructionIntrospection))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eligibility_mode_validator_accepts_expected_values() {
        assert!(validate_eligibility_mode(0).is_ok());
        assert!(validate_eligibility_mode(1).is_ok());
        assert!(validate_eligibility_mode(2).is_ok());
        assert!(validate_eligibility_mode(3).is_err());
    }

    #[test]
    fn body_validation_respects_boundaries() {
        assert!(validate_body("a", 1).is_ok());
        assert!(validate_body("", 1).is_err());
        assert!(validate_body("ab", 1).is_err());
    }

    #[test]
    fn reaction_kind_validation_respects_boundaries() {
        assert!(validate_reaction_kind("a").is_ok());
        assert!(validate_reaction_kind("").is_err());
        assert!(validate_reaction_kind(&"x".repeat(MAX_REACTION_KIND_LEN)).is_ok());
        assert!(validate_reaction_kind(&"x".repeat(MAX_REACTION_KIND_LEN + 1)).is_err());
    }

    #[test]
    fn user_eligibility_matrix_works() {
        let source_program_id = Pubkey::new_unique();
        let user = Pubkey::new_unique();

        let strict_ok = UserMetaStandard {
            source_program_id,
            user,
            is_registered: true,
            is_eligible_to_comment: true,
        };
        let registered_only = UserMetaStandard {
            source_program_id,
            user,
            is_registered: true,
            is_eligible_to_comment: false,
        };

        assert!(validate_user_eligibility(&strict_ok, source_program_id, user, 0).is_ok());
        assert!(validate_user_eligibility(&registered_only, source_program_id, user, 0).is_err());
        assert!(validate_user_eligibility(&registered_only, source_program_id, user, 1).is_ok());
        assert!(validate_user_eligibility(&registered_only, source_program_id, user, 2).is_ok());
    }

    #[test]
    fn canonical_pda_helper_rejects_wrong_bump() {
        let seed_a = b"thread";
        let seed_b = Pubkey::new_unique();
        let seeds: [&[u8]; 2] = [seed_a, seed_b.as_ref()];
        let (pda, bump) = Pubkey::find_program_address(&seeds, &crate::ID);
        assert!(assert_canonical_pda(&pda, &seeds, bump).is_ok());
        assert!(assert_canonical_pda(&pda, &seeds, bump.wrapping_add(1)).is_err());
    }
}
