pub mod access_control;
pub mod error;
pub mod instructions;
pub mod libraries;
pub mod states;
pub mod util;

use crate::access_control::*;
use crate::error::ErrorCode;
use crate::libraries::tick_math;
use anchor_lang::prelude::*;
use instructions::*;
use states::*;

declare_id!("EwgWHHfU1rbczvaJzZR8J3hB4J6yup3ZY6uCP5FCgxKU");

pub mod admin {
    use anchor_lang::prelude::declare_id;
    declare_id!("E1CjPgJodwDyAzvHEvHGcGqNeJc2ZLSe8kdJC3affmZf");
}

#[program]
pub mod amm_core {

    use super::*;

    // ---------------------------------------------------------------------
    // Factory instructions

    // The Factory facilitates creation of pools and control over the protocol fees

    /// Initialize the factory state and set the protocol owner
    ///
    /// # Arguments
    ///
    /// * `ctx`- Initializes the factory state account
    /// * `amm_config_bump` - Bump to validate factory state address
    ///
    pub fn create_amm_config(ctx: Context<CreateAmmConfig>, protocol_fee_rate: u32) -> Result<()> {
        assert!(protocol_fee_rate > 0 && protocol_fee_rate <= FEE_RATE_DENOMINATOR_VALUE);
        instructions::create_amm_config(ctx, protocol_fee_rate)
    }

    /// Updates the owner of the factory
    /// Must be called by the current owner
    ///
    /// # Arguments
    ///
    /// * `ctx`- Checks whether protocol owner has signed
    ///
    pub fn set_new_owner(ctx: Context<SetNewOwner>) -> Result<()> {
        instructions::set_new_owner(ctx)
    }

    /// Create a fee account with the given tick_spacing
    /// Fee account may never be removed once created
    ///
    /// # Arguments
    ///
    /// * `ctx`- Checks whether protocol owner has signed and initializes the fee account
    /// * `fee_state_bump` - Bump to validate fee state address
    /// * `fee` - The fee amount to enable, denominated in hundredths of a bip (i.e. 1e-6)
    /// * `tick_spacing` - The spacing between ticks to be enforced for all pools created
    /// with the given fee amount
    ///
    pub fn create_fee_account(
        ctx: Context<CreateFeeAccount>,
        fee: u32,
        tick_spacing: u16,
    ) -> Result<()> {
        instructions::create_fee_account(ctx, fee, tick_spacing)
    }

    // ---------------------------------------------------------------------
    // Pool instructions

    /// Creates a pool for the given token pair and fee, and sets the initial price
    ///
    /// A single function in place of Uniswap's Factory.createPool(), PoolDeployer.deploy()
    /// Pool.initialize() and pool.Constructor()
    ///
    /// # Arguments
    ///
    /// * `ctx`- Validates token addresses and fee state. Initializes pool, observation and
    /// token accounts
    /// * `pool_state_bump` - Bump to validate Pool State address
    /// * `observation_state_bump` - Bump to validate Observation State address
    /// * `sqrt_price_x64` - the initial sqrt price (amount_token_1 / amount_token_0) of the pool as a Q32.32
    ///
    pub fn create_pool(ctx: Context<CreatePool>, sqrt_price_x64: u128) -> Result<()> {
        instructions::create_pool(ctx, sqrt_price_x64)
    }

    /// Reset a pool sqrt price
    ///
    /// # Arguments
    ///
    /// * `ctx`- Validates token addresses and fee state. Initializes pool, observation and
    /// token accounts
    /// * `sqrt_price_x64` - the initial sqrt price (amount_token_1 / amount_token_0) of the pool as a Q32.32
    ///
    pub fn reset_sqrt_price(ctx: Context<ResetSqrtPrice>, sqrt_price_x64: u128) -> Result<()> {
        instructions::reset_sqrt_price(ctx, sqrt_price_x64)
    }
    /// Initialize a reward info for a given pool and reward index
    ///
    ///
    /// # Arguments
    ///
    /// * `ctx`- Validates token addresses and fee state. Initializes pool, observation and
    /// token accounts
    /// * `reward_index` - the index to init info
    /// * `open_time` - reward open timestamp
    /// * `end_time` - reward end timestamp
    /// * `reward_per_second` - Token reward per second are earned per unit of liquidity.
    ///
    pub fn initialize_reward(
        ctx: Context<InitializeReward>,
        param: InitializeRewardParam,
    ) -> Result<()> {
        instructions::initialize_reward(ctx, param)
    }

    /// Update fee and rewards owned.
    ///
    // #[access_control(is_authorized_for_token(&ctx.accounts.owner_or_delegate, &ctx.accounts.nft_account))]
    pub fn update_reward_infos<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, UpdateRewardInfos<'info>>,
    ) -> Result<()> {
        instructions::update_reward_infos(ctx)
    }

    /// Collects up to a derired amount of reward owed to a specific tokenized position to the recipient
    ///
    /// # Arguments
    ///
    /// * `ctx` - Validated addresses of the tokenized position and token accounts. Reward can be sent
    /// to third parties
    /// * `reward_index` - The index of reward token in the pool.
    /// * `amount_desired` - The desired amount of reward to collect, if set 0, all will be collect.
    ///
    #[access_control(is_authorized_for_token(&ctx.accounts.owner_or_delegate, &ctx.accounts.nft_account))]
    pub fn collect_rewards<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, CollectRewards<'info>>,
    ) -> Result<()> {
        instructions::collect_rewards(ctx)
    }

    /// Set reward emission per second.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Validated addresses of the tokenized position and token accounts. Reward can be sent
    /// to third parties
    /// * `reward_index` - The index of reward token in the pool.
    /// * `emissions_per_second_x64` - The per second emission reward
    ///
    pub fn set_reward_emissions<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, SetRewardEmissions<'info>>,
        reward_index: u8,
        emissions_per_second_x64: u128,
    ) -> Result<()> {
        instructions::set_reward_emissions(ctx, reward_index, emissions_per_second_x64)
    }
    // ---------------------------------------------------------------------
    // Oracle

    /// Increase the maximum number of price and liquidity observations that this pool will store
    ///
    /// An `ObservationState` account is created per unit increase in cardinality_next,
    /// and `observation_cardinality_next` is accordingly incremented.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Holds the pool and payer addresses, along with a vector of
    /// observation accounts which will be initialized
    /// * `observation_account_bumps` - Vector of bumps to initialize the observation state PDAs
    ///
    pub fn increase_observation_cardinality_next<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, IncreaseObservationCardinalityNextCtx<'info>>,
        observation_account_bumps: Vec<u8>,
    ) -> Result<()> {
        instructions::increase_observation_cardinality_next(ctx, observation_account_bumps)
    }

    // ---------------------------------------------------------------------
    // Pool owner instructions

    /// Set the denominator of the protocol's % share of the fees.
    ///
    /// Unlike Uniswap, protocol fee is globally set. It can be updated by factory owner
    /// at any time.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Checks for valid owner by looking at signer and factory owner addresses.
    /// Holds the Factory State account where protocol fee will be saved.
    /// * `fee_protocol` - new protocol fee for all pools
    ///
    pub fn set_protocol_fee_rate(
        ctx: Context<SetProtocolFeeRate>,
        protocol_fee_rate: u32,
    ) -> Result<()> {
        assert!(protocol_fee_rate > 0 && protocol_fee_rate <= FEE_RATE_DENOMINATOR_VALUE);
        let amm_config = &mut ctx.accounts.amm_config;
        let protocol_fee_rate_old = amm_config.protocol_fee_rate;
        amm_config.protocol_fee_rate = protocol_fee_rate;

        emit!(SetProtocolFeeRateEvent {
            protocol_fee_rate_old,
            protocol_fee_rate_new: protocol_fee_rate
        });

        Ok(())
    }

    /// Collect the protocol fee accrued to the pool
    ///
    /// # Arguments
    ///
    /// * `ctx` - Checks for valid owner by looking at signer and factory owner addresses.
    /// Holds the Pool State account where accrued protocol fee is saved, and token accounts to perform
    /// transfer.
    /// * `amount_0_requested` - The maximum amount of token_0 to send, can be 0 to collect fees in only token_1
    /// * `amount_1_requested` - The maximum amount of token_1 to send, can be 0 to collect fees in only token_0
    ///
    pub fn collect_protocol_fee(
        ctx: Context<CollectProtocolFee>,
        amount_0_requested: u64,
        amount_1_requested: u64,
    ) -> Result<()> {
        instructions::collect_protocol_fee(ctx, amount_0_requested, amount_1_requested)
    }

    // ---------------------------------------------------------------------
    // Position instructions

    // Non fungible position manager

    /// Creates a new position wrapped in a NFT
    ///
    /// # Arguments
    ///
    /// * `ctx` - Holds pool, tick, bitmap, position and token accounts
    /// * `amount_0_desired` - Desired amount of token_0 to be spent
    /// * `amount_1_desired` - Desired amount of token_1 to be spent
    /// * `amount_0_min` - The minimum amount of token_0 to spend, which serves as a slippage check
    /// * `amount_1_min` - The minimum amount of token_1 to spend, which serves as a slippage check
    /// * `deadline` - The time by which the transaction must be included to effect the change
    ///
    pub fn create_position<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, CreatePosition<'info>>,
        tick_lower: i32,
        tick_upper: i32,
        word_pos_lower: i16,
        word_pos_upper: i16,
        amount_0_desired: u64,
        amount_1_desired: u64,
        amount_0_min: u64,
        amount_1_min: u64,
    ) -> Result<()> {
        instructions::create_position(
            ctx,
            amount_0_desired,
            amount_1_desired,
            amount_0_min,
            amount_1_min,
            tick_lower,
            tick_upper,
            word_pos_lower,
            word_pos_upper,
        )
    }
   
    /// Increases liquidity in a tokenized position, with amount paid by `payer`
    ///
    /// # Arguments
    ///
    /// * `ctx` - Holds the pool, tick, bitmap, position and token accounts
    /// * `amount_0_desired` - Desired amount of token_0 to be spent
    /// * `amount_1_desired` - Desired amount of token_1 to be spent
    /// * `amount_0_min` - The minimum amount of token_0 to spend, which serves as a slippage check
    /// * `amount_1_min` - The minimum amount of token_1 to spend, which serves as a slippage check
    /// * `deadline` - The time by which the transaction must be included to effect the change
    ///
    pub fn increase_liquidity<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, IncreaseLiquidity<'info>>,
        amount_0_desired: u64,
        amount_1_desired: u64,
        amount_0_min: u64,
        amount_1_min: u64,
    ) -> Result<()> {
        instructions::increase_liquidity(
            ctx,
            amount_0_desired,
            amount_1_desired,
            amount_0_min,
            amount_1_min,
        )
    }
    /// Decreases the amount of liquidity in a position and accounts it to the position
    ///
    /// # Arguments
    ///
    /// * `ctx` - Holds the pool, tick, bitmap, position and token accounts
    /// * `liquidity` - The amount by which liquidity will be decreased
    /// * `amount_0_min` - The minimum amount of token_0 that should be accounted for the burned liquidity
    /// * `amount_1_min` - The minimum amount of token_1 that should be accounted for the burned liquidity
    /// * `deadline` - The time by which the transaction must be included to effect the change
    ///
    #[access_control(is_authorized_for_token(&ctx.accounts.owner_or_delegate, &ctx.accounts.nft_account))]
    pub fn decrease_liquidity<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, DecreaseLiquidity<'info>>,
        liquidity: u128,
        amount_0_min: u64,
        amount_1_min: u64,
    ) -> Result<()> {
        instructions::decrease_liquidity(ctx, liquidity, amount_0_min, amount_1_min)
    }

    /// Collects up to a maximum amount of fees owed to a specific tokenized position to the recipient
    ///
    /// # Arguments
    ///
    /// * `ctx` - Validated addresses of the tokenized position and token accounts. Fees can be sent
    /// to third parties
    /// * `amount_0_max` - The maximum amount of token0 to collect
    /// * `amount_1_max` - The maximum amount of token0 to collect
    ///
    #[access_control(is_authorized_for_token(&ctx.accounts.owner_or_delegate, &ctx.accounts.nft_account))]
    pub fn collect_fee<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, CollectFee<'info>>,
        amount_0_max: u64,
        amount_1_max: u64,
    ) -> Result<()> {
        instructions::collect_fee(ctx, amount_0_max, amount_1_max)
    }

    /// Swaps `amount_in` of one token for as much as possible of another token,
    /// across a single pool
    ///
    /// # Arguments
    ///
    /// * `ctx` - Accounts required for the swap
    /// * `deadline` - The time by which the transaction must be included to effect the change
    /// * `amount_in` - Arranged in pairs with other_amount_threshold. (amount_in, amount_out_minimum) or (amount_out, amount_in_maximum)
    /// * `other_amount_threshold` - For slippage check
    /// * `sqrt_price_limit` - The Q32.32 sqrt price √P limit. If zero for one, the price cannot
    /// be less than this value after the swap.  If one for zero, the price cannot be greater than
    /// this value after the swap.
    ///
    pub fn swap<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, SwapSingle<'info>>,
        amount: u64,
        other_amount_threshold: u64,
        sqrt_price_limit_x64: u128,
        is_base_input: bool,
    ) -> Result<()> {
        instructions::swap(
            ctx,
            amount,
            other_amount_threshold,
            sqrt_price_limit_x64,
            is_base_input,
        )
    }
    /// Swaps `amount_in` of one token for as much as possible of another token,
    /// across the path provided
    ///
    /// # Arguments
    ///
    /// * `ctx` - Accounts for token transfer and swap route
    /// * `deadline` - Swap should if fail if past deadline
    /// * `amount_in` - Token amount to be swapped in
    /// * `amount_out_minimum` - Panic if output amount is below minimum amount. For slippage.
    /// * `additional_accounts_per_pool` - Additional observation, bitmap and tick accounts per pool
    ///
    pub fn swap_router_base_in<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, SwapBaseIn<'info>>,
        amount_in: u64,
        amount_out_minimum: u64,
        additional_accounts_per_pool: Vec<u8>,
    ) -> Result<()> {
        instructions::swap_router_base_in(
            ctx,
            amount_in,
            amount_out_minimum,
            additional_accounts_per_pool,
        )
    }

    // /// Swaps as little as possible of one token for `amount_out` of another
    // /// along the specified path (reversed)
    // ///
    // /// # Arguments
    // ///
    // /// * `ctx` - Accounts for token transfer and swap route
    // /// * `deadline` - Swap should if fail if past deadline
    // /// * `amount_out` - Token amount to be swapped out
    // /// * `amount_in_maximum` - For slippage. Panic if required input exceeds max limit.
    // ///
    // pub fn exact_output(
    //     ctx: Context<ExactInput>,
    //     deadline: u64,
    //     amount_out: u64,
    //     amount_out_maximum: u64,
    // ) -> Result<()> {
    //     todo!()
    // }
}

/// Common checks for a valid tick input.
/// A tick is valid iff it lies within tick boundaries and it is a multiple
/// of tick spacing.
///
/// # Arguments
///
/// * `tick` - The price tick
///
pub fn check_tick(tick: i32, tick_spacing: u16) -> Result<()> {
    require!(tick >= tick_math::MIN_TICK, ErrorCode::TickLowerOverflow);
    require!(tick <= tick_math::MAX_TICK, ErrorCode::TickUpperOverflow);
    require!(
        tick % tick_spacing as i32 == 0,
        ErrorCode::TickAndSpacingNotMatch
    );
    Ok(())
}

/// Common checks for valid tick inputs.
///
/// # Arguments
///
/// * `tick_lower` - The lower tick
/// * `tick_upper` - The upper tick
///
pub fn check_ticks(tick_lower: i32, tick_upper: i32) -> Result<()> {
    require!(tick_lower < tick_upper, ErrorCode::TickInvaildOrder);
    Ok(())
}