# Pallet Staking Fork Notes

We forked [pallet_staking](https://github.com/paritytech/substrate/tree/v3.0.0/frame/staking) and made following modifications:

### Removed reward curve

We removed the reward curve configuration on the pallet because we are not implementing traditional inflation for the rewards of validators.

### Staking rewards

The staking pallet is configured to reward a total of 1% of a reward pool after each era to the validators of that era.   

### Staking Pool Account

The `staking_pool_account` is a configurable trait that holds the rewards to be paid out to the validators/stakers. 
Threefold will be sponsoring this account to give incentive to users to start staking or run validators.

### Staking Reward Account

The `staking_reward_account` is a configurable trait this is an intermediate account where 1% of the total balance of the `staking_pool_account` is sent to after each era. 
When validators claim their rewards, the respective amounts are sent from this account to the validators.

The reason we work with an intermediate account is that the 1% rewards after each era based on the total balance of the `staking_pool_account` are correct every era.

To support this, the `end_era` function has been modified to transfer 1% of the `staking_pool_account` to the `staking_reward_account`.

For example, 

Era 1: `staking_pool_account` has a total balance of 1.000.000 TFT. After that era 1% of that amount is send to the `staking_reward_account` so 990.000 is left on the `staking_pool_account`. 
So the next era reward calculation would be 1% of 990.000. If we would not use an intermedia account, there could be a possibility that rewards were unclaimed and the next era calculation would be wrong (since the tokens did not get transfered to the validators).