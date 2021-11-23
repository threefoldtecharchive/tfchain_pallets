# Pallet Staking Fork Notes

We forked [pallet_staking](https://github.com/paritytech/substrate/tree/v3.0.0/frame/staking) and made following modifications:

### Removed reward curve

We removed the reward curve configuration on the pallet because we are not implementing traditional inflation for the rewards of validators.

### Staking Pool Account

A `staking_pool_account` storage map has been added to contain the address where the staking rewards are withdrawn from in order to payout validators

To support this, the `end_era` function has been modified to payout 1% of the total balance of the stakingpool account to the validators each era.