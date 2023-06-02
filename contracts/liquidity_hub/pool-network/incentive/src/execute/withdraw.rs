use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError, Uint128};
use white_whale::pool_network::asset::Asset;

use crate::state::ADDRESS_WEIGHT_HISTORY;
use crate::{
    error::ContractError,
    state::{ADDRESS_WEIGHT, CLOSED_POSITIONS, CONFIG, GLOBAL_WEIGHT},
};

pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    // counter of how many LP tokens we must return to use and the weight to remove
    let mut return_token_count = Uint128::zero();

    println!("withdraw, user {}", info.sender.clone());

    CLOSED_POSITIONS.update::<_, ContractError>(
        deps.storage,
        info.sender.clone(),
        |closed_positions| {
            let mut closed_positions = closed_positions.unwrap_or_default();
            println!("closed_positions: {:?}", closed_positions);

            for i in (0..closed_positions.len()).rev() {
                let position = &closed_positions[i];
                println!("env.block.time.seconds(): {:?}", env.block.time.seconds());
                println!(
                    "position.unbonding_timestamp(): {:?}",
                    position.unbonding_timestamp
                );
                if env.block.time.seconds() > position.unbonding_timestamp {
                    // todo remove
                    // // remove weight
                    // // this should be the position amount, as that is the amount we didn't subtract
                    // // when we closed the position
                    // weight_to_remove = weight_to_remove.checked_add(position.amount)?;

                    // add return tokens to sum
                    return_token_count = return_token_count.checked_add(position.amount)?;

                    // remove position
                    closed_positions.remove(i);
                }
            }

            Ok(closed_positions)
        },
    )?;

    // todo remove, the weight stuff has already been taken care of when closing the position
    // if !weight_to_remove.is_zero() {
    //     GLOBAL_WEIGHT.update::<_, StdError>(deps.storage, |global_weight| {
    //         Ok(global_weight.checked_sub(weight_to_remove)?)
    //     })?;
    //     // ADDRESS_WEIGHT.update::<_, StdError>(deps.storage, info.sender.clone(), |user_weight| {
    //     //     Ok(user_weight
    //     //         .unwrap_or_default()
    //     //         .checked_sub(weight_to_remove)?)
    //     // })?;
    //
    //     let config = CONFIG.load(deps.storage)?;
    //     let epoch_response: white_whale::fee_distributor::EpochResponse =
    //         deps.querier.query_wasm_smart(
    //             config.fee_distributor_address.into_string(),
    //             &white_whale::fee_distributor::QueryMsg::CurrentEpoch {},
    //         )?;
    //
    //     let mut user_weight = ADDRESS_WEIGHT
    //         .may_load(deps.storage, info.sender.clone())?
    //         .unwrap_or_default();
    //     user_weight = user_weight.checked_sub(weight_to_remove)?;
    //
    //     ADDRESS_WEIGHT_HISTORY.update::<_, StdError>(
    //         deps.storage,
    //         (&info.sender.clone(), epoch_response.epoch.id.u64() + 1u64),
    //         |_| Ok(user_weight),
    //     )?;
    //
    //     ADDRESS_WEIGHT.save(deps.storage, info.sender.clone(), &user_weight)?;
    // }

    println!("withdraw, return_token_count {}", return_token_count);

    if !return_token_count.is_zero() {
        let config = CONFIG.load(deps.storage)?;

        let return_asset = Asset {
            info: config.lp_asset,
            amount: return_token_count.clone(),
        };

        println!("withdraw, return_asset {:?}", return_asset);

        return Ok(Response::default()
            .add_attributes(vec![
                ("action", "withdraw".to_string()),
                ("return_asset", return_asset.to_string()),
            ])
            .add_message(return_asset.into_msg(info.sender)?));
    }

    // there was no positions we closed
    Ok(Response::default().add_attributes(vec![
        ("action", "withdraw"),
        ("result", "no positions were closed"),
    ]))
}
