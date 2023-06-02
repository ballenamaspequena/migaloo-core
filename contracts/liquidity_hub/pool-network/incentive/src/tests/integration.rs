use std::cell::RefCell;
use std::error::Error;

use cosmwasm_std::{coin, coins, Addr, StdError, Timestamp, Uint128};

use white_whale::pool_network::asset::{Asset, AssetInfo};
use white_whale::pool_network::incentive;
use white_whale::pool_network::incentive::{Curve, Flow};
use white_whale::pool_network::incentive_factory::{
    IncentiveResponse, IncentivesContract, IncentivesResponse,
};

use crate::error::ContractError;
use crate::tests::suite::TestingSuite;

#[test]
fn instantiate_incentive_factory_successful() {
    let mut suite = TestingSuite::default();

    suite.instantiate(
        "fee_collector_addr".to_string(),
        "fee_distributor_addr".to_string(),
        Asset {
            amount: Uint128::new(1_000u128),
            info: AssetInfo::NativeToken {
                denom: "uwhale".to_string(),
            },
        },
        1_000,
        1,
        1_000,
        1_000,
        2_000,
    );
}

#[test]
fn instantiate_incentive_factory_unsuccessful() {
    let mut suite = TestingSuite::default();

    suite.instantiate_err(
        "fee_collector_addr".to_string(),
        "fee_distributor_addr".to_string(),
        Asset {
            amount: Uint128::new(1_000u128),
            info: AssetInfo::NativeToken {
                denom: "uwhale".to_string(),
            },
        },
        1_000,
        1,
        1_000,
        1_000,
        500,
        |error| {
            let err = error
                .downcast::<incentive_factory::error::ContractError>()
                .unwrap();

            match err {
                incentive_factory::error::ContractError::InvalidUnbondingRange { min, max } => {
                    assert_eq!(min, 1000);
                    assert_eq!(max, 500);
                }
                _ => panic!("Wrong error type, should return ContractError::InvalidUnbondingRange"),
            }
        },
    );
}

#[test]
fn create_incentive_cw20_lp_with_duplicate() {
    let mut suite =
        TestingSuite::default_with_balances(coins(1_000_000_000u128, "uwhale".to_string()));
    let creator = suite.creator();
    let unauthorized = suite.senders[2].clone();

    suite.instantiate_default_native_fee().create_lp_tokens();

    let lp_asset_1 = AssetInfo::Token {
        contract_addr: suite.cw20_tokens.first().unwrap().to_string(),
    };
    let lp_asset_2 = AssetInfo::Token {
        contract_addr: suite.cw20_tokens.last().unwrap().to_string(),
    };

    let lp_assets: Vec<AssetInfo> = vec![lp_asset_1.clone(), lp_asset_2.clone()];
    let mut incentives: RefCell<Vec<IncentivesContract>> = RefCell::new(vec![]);

    // first try to execute anything on the incentive factory contract from a non-owner, it should error
    // then do it with the owner of the contract
    suite
        .create_incentive(unauthorized, lp_asset_1.clone(), |result| {
            let err = result
                .unwrap_err()
                .downcast::<incentive_factory::error::ContractError>()
                .unwrap();

            match err {
                incentive_factory::error::ContractError::Unauthorized {} => {}
                _ => panic!("Wrong error type, should return ContractError::Unauthorized"),
            }
        })
        .create_incentive(creator.clone(), lp_asset_1.clone(), |result| {
            result.unwrap();
        })
        .query_incentive(lp_asset_1.clone(), |result| {
            let incentive = result.unwrap();
            assert!(incentive.is_some());
        })
        // this should error cuz the incentive for that lp was already created
        .create_incentive(creator.clone(), lp_asset_1.clone(), |result| {
            let err = result
                .unwrap_err()
                .downcast::<incentive_factory::error::ContractError>()
                .unwrap();

            match err {
                incentive_factory::error::ContractError::DuplicateIncentiveContract { .. } => {}
                _ => panic!(
                    "Wrong error type, should return ContractError::DuplicateIncentiveContract"
                ),
            }
        })
        .create_incentive(creator.clone(), lp_asset_2, |result| {
            result.unwrap();
        })
        .query_incentives(None, None, |result| {
            let incentives_response = result.unwrap();
            assert_eq!(incentives_response.len(), 2usize);
            *incentives.borrow_mut() = incentives_response;
        })
        .query_incentive_config(
            incentives
                .clone()
                .into_inner()
                .first()
                .unwrap()
                .incentive_address
                .clone(),
            |result| {
                let config = result.unwrap();
                assert_eq!(config.lp_asset, lp_assets.first().unwrap().clone());
            },
        )
        .query_incentive_config(
            incentives
                .clone()
                .into_inner()
                .last()
                .unwrap()
                .incentive_address
                .clone(),
            |result| {
                let config = result.unwrap();
                assert_eq!(config.lp_asset, lp_assets.last().unwrap().clone());
            },
        );
}

#[test]
fn create_incentive_native_lp_with_duplicate() {
    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uwhale".to_string()),
        coin(1_000_000_000u128, "factory/creator/uLP".to_string()),
        coin(1_000_000_000u128, "factory/another_creator/uLP".to_string()),
    ]);
    let creator = suite.creator();
    let unauthorized = suite.senders[2].clone();

    suite.instantiate_default_native_fee().create_lp_tokens();

    let lp_asset_1 = AssetInfo::NativeToken {
        denom: "factory/creator1/uLP".to_string(),
    };
    let lp_asset_2 = AssetInfo::NativeToken {
        denom: "factory/creator2/uLP".to_string(),
    };

    let lp_assets: Vec<AssetInfo> = vec![lp_asset_1.clone(), lp_asset_2.clone()];
    let mut incentives: RefCell<Vec<IncentivesContract>> = RefCell::new(vec![]);

    // first try to execute anything on the incentive factory contract from a non-owner, it should error
    // then do it with the owner of the contract
    suite
        .create_incentive(unauthorized, lp_asset_1.clone(), |result| {
            let err = result
                .unwrap_err()
                .downcast::<incentive_factory::error::ContractError>()
                .unwrap();

            match err {
                incentive_factory::error::ContractError::Unauthorized {} => {}
                _ => panic!("Wrong error type, should return ContractError::Unauthorized"),
            }
        })
        .create_incentive(creator.clone(), lp_asset_1.clone(), |result| {
            result.unwrap();
        })
        .query_incentive(lp_asset_1.clone(), |result| {
            let incentive = result.unwrap();
            assert!(incentive.is_some());
        })
        // this should error cuz the incentive for that lp was already created
        .create_incentive(creator.clone(), lp_asset_1.clone(), |result| {
            let err = result
                .unwrap_err()
                .downcast::<incentive_factory::error::ContractError>()
                .unwrap();

            match err {
                incentive_factory::error::ContractError::DuplicateIncentiveContract { .. } => {}
                _ => panic!(
                    "Wrong error type, should return ContractError::DuplicateIncentiveContract"
                ),
            }
        })
        .create_incentive(creator.clone(), lp_asset_2, |result| {
            result.unwrap();
        })
        .query_incentives(None, None, |result| {
            let incentives_response = result.unwrap();
            assert_eq!(incentives_response.len(), 2usize);
            *incentives.borrow_mut() = incentives_response;
        })
        .query_incentive_config(
            incentives
                .clone()
                .into_inner()
                .first()
                .unwrap()
                .incentive_address
                .clone(),
            |result| {
                let config = result.unwrap();
                assert_eq!(config.lp_asset, lp_assets.first().unwrap().clone());
            },
        )
        .query_incentive_config(
            incentives
                .clone()
                .into_inner()
                .last()
                .unwrap()
                .incentive_address
                .clone(),
            |result| {
                let config = result.unwrap();
                assert_eq!(config.lp_asset, lp_assets.last().unwrap().clone());
            },
        );
}

#[test]
fn try_open_more_flows_than_allowed() {
    let mut suite =
        TestingSuite::default_with_balances(vec![coin(1_000_000_000u128, "uwhale".to_string())]);
    let alice = suite.creator();

    suite.instantiate_default_native_fee().create_lp_tokens();

    let lp_address_1 = AssetInfo::Token {
        contract_addr: suite.cw20_tokens.first().unwrap().to_string(),
    };

    let mut incentive_addr = RefCell::new(Addr::unchecked(""));

    suite
        .create_incentive(alice.clone(), lp_address_1.clone(), |result| {
            result.unwrap();
        })
        .query_incentive(lp_address_1.clone(), |result| {
            let incentive = result.unwrap();
            assert!(incentive.is_some());
            *incentive_addr.borrow_mut() = incentive.unwrap();
        });

    // open 7 incentives, it should fail on the 8th
    let app_time = suite.get_time();

    for i in 1..=8 {
        suite.open_incentive_flow(
            alice.clone(),
            incentive_addr.clone().into_inner(),
            None,
            1884342800u64,
            None,
            10u64,
            Curve::Linear,
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uwhale".to_string(),
                },
                amount: Uint128::new(i * 2_000u128),
            },
            &vec![coin(i * 2_000u128, "uwhale".to_string())],
            |result| {
                if i > 7 {
                    // this should fail as only 7 incentives can be opened as specified in `instantiate_default`
                    let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                    match err {
                        ContractError::TooManyFlows { .. } => {}
                        _ => panic!("Wrong error type, should return ContractError::TooManyFlows"),
                    }
                } else {
                    result.unwrap();
                }
            },
        );
    }

    let mut incentive_flows = RefCell::new(vec![]);
    suite.query_flows(incentive_addr.clone().into_inner(), |result| {
        let flows = result.unwrap();

        *incentive_flows.borrow_mut() = flows.clone();

        assert_eq!(flows.len(), 7usize);
        assert_eq!(
            flows.first().unwrap(),
            &Flow {
                flow_id: 1,
                flow_creator: alice.clone(),
                flow_asset: Asset {
                    info: AssetInfo::NativeToken {
                        denom: "uwhale".to_string(),
                    },
                    amount: Uint128::new(1_000u128),
                },
                claimed_amount: Uint128::zero(),
                curve: Curve::Linear,
                start_timestamp: app_time.clone().seconds(),
                end_timestamp: app_time.clone().seconds(),
                start_epoch: 1u64,
                end_epoch: 10u64,
                emissions_per_epoch: Default::default(),
                emitted_tokens: Default::default(),
            }
        );
        assert_eq!(
            flows.last().unwrap(),
            &Flow {
                flow_id: 7,
                flow_creator: alice.clone(),
                flow_asset: Asset {
                    info: AssetInfo::NativeToken {
                        denom: "uwhale".to_string(),
                    },
                    amount: Uint128::new(13_000u128),
                },
                claimed_amount: Uint128::zero(),
                curve: Curve::Linear,
                start_timestamp: app_time.clone().seconds(),
                end_timestamp: app_time.clone().seconds(),
                start_epoch: 1u64,
                end_epoch: 10u64,
                emissions_per_epoch: Default::default(),
                emitted_tokens: Default::default(),
            }
        );
    });
}

#[test]
fn try_open_flows_with_wrong_epochs() {
    let mut suite =
        TestingSuite::default_with_balances(vec![coin(1_000_000_000u128, "uwhale".to_string())]);
    let alice = suite.creator();

    suite.instantiate_default_native_fee().create_lp_tokens();

    let lp_address_1 = AssetInfo::Token {
        contract_addr: suite.cw20_tokens.first().unwrap().to_string(),
    };

    let mut incentive_addr = RefCell::new(Addr::unchecked(""));
    let mut max_flow_epoch_buffer = RefCell::new(0u64);

    suite
        .create_incentive(alice.clone(), lp_address_1.clone(), |result| {
            result.unwrap();
        })
        .query_incentive(lp_address_1.clone(), |result| {
            let incentive = result.unwrap();
            assert!(incentive.is_some());
            *incentive_addr.borrow_mut() = incentive.unwrap();
        });

    // open incentive flow
    //todo remove these future timestamps as epochs are used
    let app_time = suite.get_time();
    let future_time = app_time.clone().plus_seconds(604800u64);
    let future_future_time = future_time.clone().plus_seconds(907200u64);
    let past_time = app_time.clone().minus_seconds(86400u64);

    let mut current_epoch = RefCell::new(0u64);
    suite
        .create_epochs_on_fee_distributor(9u64, vec![])
        .query_current_epoch(|result| {
            *current_epoch.borrow_mut() = result.unwrap().epoch.id.u64();
        });

    let future_epoch = current_epoch.clone().into_inner() + 5u64;
    let future_future_epoch = current_epoch.clone().into_inner() + 10u64;
    let past_epoch = current_epoch.clone().into_inner() - 5u64;

    suite
        .query_incentive_factory_config(|result| {
            let config = result.unwrap();
            *max_flow_epoch_buffer.borrow_mut() = config.max_flow_epoch_buffer;
        })
        .open_incentive_flow(
            alice.clone(),
            incentive_addr.clone().into_inner(),
            None,
            past_time.clone().seconds(),
            None,
            past_epoch.clone(),
            Curve::Linear,
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uwhale".clone().to_string(),
                },
                amount: Uint128::new(2_000u128),
            },
            &vec![coin(2_000u128, "uwhale".to_string())],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::FlowExpirationInPast {} => {}
                    _ => panic!(
                        "Wrong error type, should return ContractError::FlowExpirationInPast"
                    ),
                }
            },
        )
        .open_incentive_flow(
            alice.clone(),
            incentive_addr.clone().into_inner(),
            Some(future_future_time.clone().seconds()),
            future_time.clone().seconds(),
            Some(future_future_epoch.clone()),
            future_epoch.clone(),
            Curve::Linear,
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uwhale".clone().to_string(),
                },
                amount: Uint128::new(2_000u128),
            },
            &vec![coin(2_000u128, "uwhale".to_string())],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::FlowStartTimeAfterEndTime {} => {}
                    _ => panic!(
                        "Wrong error type, should return ContractError::FlowStartTimeAfterEndTime"
                    ),
                }
            },
        )
        .open_incentive_flow(
            alice.clone(),
            incentive_addr.clone().into_inner(),
            Some(
                app_time
                    .clone()
                    .plus_seconds(max_flow_epoch_buffer.clone().into_inner() + 1)
                    .seconds(),
            ),
            app_time
                .clone()
                .plus_seconds(max_flow_epoch_buffer.clone().into_inner() + 1000)
                .seconds(),
            Some(
                current_epoch.clone().into_inner() + max_flow_epoch_buffer.clone().into_inner() + 1,
            ),
            current_epoch.clone().into_inner() + 100,
            Curve::Linear,
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uwhale".clone().to_string(),
                },
                amount: Uint128::new(2_000u128),
            },
            &vec![coin(2_000u128, "uwhale".to_string())],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::FlowStartTooFar {} => {}
                    _ => panic!("Wrong error type, should return ContractError::FlowStartTooFar"),
                }
            },
        )
        .open_incentive_flow(
            alice.clone(),
            incentive_addr.clone().into_inner(),
            None,
            future_time.clone().seconds(),
            None,
            future_epoch.clone(),
            Curve::Linear,
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uwhale".clone().to_string(),
                },
                amount: Uint128::new(2_000u128),
            },
            &vec![coin(2_000u128, "uwhale".to_string())],
            |result| {
                result.unwrap();
            },
        );
}

#[test]
fn open_flow_with_fee_native_token_and_flow_same_native_token() {
    let mut suite =
        TestingSuite::default_with_balances(vec![coin(1_000_000_000u128, "uwhale".to_string())]);
    let alice = suite.creator();
    let carol = suite.senders[2].clone();

    suite.instantiate_default_native_fee().create_lp_tokens();

    let mut fee_collector_addr = RefCell::new(Addr::unchecked(""));

    let lp_address_1 = AssetInfo::Token {
        contract_addr: suite.cw20_tokens.first().unwrap().to_string(),
    };

    let mut incentive_addr = RefCell::new(Addr::unchecked(""));

    suite
        .create_incentive(alice.clone(), lp_address_1.clone(), |result| {
            result.unwrap();
        })
        .query_incentive(lp_address_1.clone(), |result| {
            let incentive = result.unwrap();
            assert!(incentive.is_some());
            *incentive_addr.borrow_mut() = incentive.unwrap();
        });

    let mut current_epoch = RefCell::new(0u64);
    suite
        .create_epochs_on_fee_distributor(9u64, vec![])
        .query_current_epoch(|result| {
            *current_epoch.borrow_mut() = result.unwrap().epoch.id.u64();
        });

    // open incentive flow
    let app_time = suite.get_time();

    suite
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uwhale".clone().to_string(),
                },
                amount: Uint128::new(0u128),
            },
            &vec![coin(1_000u128, "uwhale".to_string())],
            |result| {
                // this should fail as not enough funds were sent
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::EmptyFlow { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::EmptyFlow"),
                }
            },
        )
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uwhale".clone().to_string(),
                },
                amount: Uint128::new(1_000u128),
            },
            &vec![coin(1_000u128, "uwhale".to_string())],
            |result| {
                // this should fail as not enough funds were sent to cover for fee + MIN_FLOW_AMOUNT
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::EmptyFlowAfterFee { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::EmptyFlowAfterFee"),
                }
            },
        )
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uwhale".clone().to_string(),
                },
                amount: Uint128::new(1_000u128),
            },
            &vec![coin(100u128, "uwhale".to_string())],
            |result| {
                // this should fail as not enough funds were sent to cover for fee + MIN_FLOW_AMOUNT
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::EmptyFlowAfterFee { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::EmptyFlowAfterFee"),
                }
            },
        )
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uwhale".clone().to_string(),
                },
                amount: Uint128::new(2_000u128),
            },
            &vec![coin(500u128, "uwhale".to_string())],
            |result| {
                // this should fail as we didn't send enough funds to cover for the fee
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::FlowFeeNotPaid { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::FlowFeeNotPaid"),
                }
            },
        )
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uwhale".clone().to_string(),
                },
                amount: Uint128::new(2_000u128),
            },
            &vec![coin(2_000u128, "uwhale".to_string())],
            |result| {
                // this should succeed as we sent enough funds to cover for fee + MIN_FLOW_AMOUNT
                result.unwrap();
            },
        )
        .query_funds(
            incentive_addr.clone().into_inner(),
            AssetInfo::NativeToken {
                denom: "uwhale".to_string(),
            },
            |funds| {
                // funds on the incentive contract
                assert_eq!(funds, Uint128::new(1_000u128));
            },
        )
        .query_incentive_factory_config(|result| {
            *fee_collector_addr.borrow_mut() = result.unwrap().fee_collector_addr;
        })
        .query_funds(
            fee_collector_addr.clone().into_inner(),
            AssetInfo::NativeToken {
                denom: "uwhale".to_string(),
            },
            |funds| {
                // funds on the fee collector
                assert_eq!(funds, Uint128::new(1_000u128));
            },
        )
        .query_flow(incentive_addr.clone().into_inner(), 1u64, |result| {
            let flow_response = result.unwrap();
            assert_eq!(
                flow_response.unwrap().flow,
                Some(Flow {
                    flow_id: 1,
                    flow_creator: carol.clone(),
                    flow_asset: Asset {
                        info: AssetInfo::NativeToken {
                            denom: "uwhale".to_string()
                        },
                        amount: Uint128::new(1_000u128),
                    },
                    claimed_amount: Uint128::zero(),
                    curve: Curve::Linear,
                    start_timestamp: app_time.clone().seconds(),
                    end_timestamp: app_time.clone().seconds(),
                    start_epoch: 10u64,
                    end_epoch: 19u64,
                    emissions_per_epoch: Default::default(),
                    emitted_tokens: Default::default(),
                })
            );
        })
        .query_flow(incentive_addr.clone().into_inner(), 5u64, |result| {
            // this should not work as there is no flow with id 5
            let flow_response = result.unwrap();
            assert_eq!(flow_response, None);
        });
}

#[test]
fn open_flow_with_fee_native_token_and_flow_different_native_token() {
    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uwhale".to_string()),
        coin(1_000_000_000u128, "ampWHALE".to_string()),
    ]);
    let alice = suite.creator();
    let carol = suite.senders[2].clone();

    suite.instantiate_default_native_fee().create_lp_tokens();

    let mut fee_collector_addr = RefCell::new(Addr::unchecked(""));

    let lp_address_1 = AssetInfo::Token {
        contract_addr: suite.cw20_tokens.first().unwrap().to_string(),
    };

    let mut incentive_addr = RefCell::new(Addr::unchecked(""));

    suite
        .create_incentive(alice.clone(), lp_address_1.clone(), |result| {
            result.unwrap();
        })
        .query_incentive(lp_address_1.clone(), |result| {
            let incentive = result.unwrap();
            assert!(incentive.is_some());
            *incentive_addr.borrow_mut() = incentive.unwrap();
        });

    let mut carol_original_uwhale_funds = RefCell::new(Uint128::zero());

    let mut current_epoch = RefCell::new(0u64);
    suite.query_current_epoch(|result| {
        *current_epoch.borrow_mut() = result.unwrap().epoch.id.u64();
    });

    // open incentive flow
    let app_time = suite.get_time();
    suite
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "ampWHALE".clone().to_string(),
                },
                amount: Uint128::new(500u128),
            },
            &vec![coin(1_000u128, "uwhale".to_string())],
            |result| {
                // this should fail as MIN_FLOW_AMOUNT is not met
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::EmptyFlow { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::EmptyFlow"),
                }
            },
        )
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "ampWHALE".clone().to_string(),
                },
                amount: Uint128::new(1_000u128),
            },
            &vec![coin(1_000u128, "uwhale".to_string())],
            |result| {
                // this should fail as the flow asset was not sent
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::FlowAssetNotSent { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::FlowAssetNotSent"),
                }
            },
        )
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "ampWHALE".clone().to_string(),
                },
                amount: Uint128::new(1_000u128),
            },
            &vec![
                coin(1_000u128, "uwhale".to_string()),
                coin(500u128, "ampWHALE".to_string()),
            ],
            |result| {
                // this should fail as the flow asset amount doesn't match the one sent to the contract
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::FlowAssetNotSent { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::FlowAssetNotSent"),
                }
            },
        )
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "ampWHALE".clone().to_string(),
                },
                amount: Uint128::new(1_000u128),
            },
            &vec![
                coin(100u128, "uwhale".to_string()),
                coin(1_00u128, "ampWHALE".to_string()),
            ],
            |result| {
                // this should fail as not enough funds were sent to cover for fee
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::FlowFeeNotPaid { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::FlowFeeNotPaid"),
                }
            },
        )
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "ampWHALE".clone().to_string(),
                },
                amount: Uint128::new(1_000u128),
            },
            &vec![
                coin(1_000u128, "uwhale".to_string()),
                coin(1_000u128, "ampWHALE".to_string()),
            ],
            |result| {
                // this should succeed as both the fee was paid in full and the flow asset amount
                // matches the one sent to the contract
                result.unwrap();
            },
        )
        .query_funds(
            incentive_addr.clone().into_inner(),
            AssetInfo::NativeToken {
                denom: "ampWHALE".to_string(),
            },
            |funds| {
                // funds on the incentive contract
                assert_eq!(funds, Uint128::new(1_000u128));
            },
        )
        .query_funds(
            incentive_addr.clone().into_inner(),
            AssetInfo::NativeToken {
                denom: "uwhale".to_string(),
            },
            |funds| {
                // no uwhale should have been sent to the incentive contract
                assert_eq!(funds, Uint128::zero());
            },
        )
        .query_incentive_factory_config(|result| {
            *fee_collector_addr.borrow_mut() = result.unwrap().fee_collector_addr;
        })
        .query_funds(
            fee_collector_addr.clone().into_inner(),
            AssetInfo::NativeToken {
                denom: "uwhale".to_string(),
            },
            |funds| {
                // funds on the fee collector
                assert_eq!(funds, Uint128::new(1_000u128));
            },
        )
        .query_funds(
            fee_collector_addr.clone().into_inner(),
            AssetInfo::NativeToken {
                denom: "ampWHALE".to_string(),
            },
            |funds| {
                // no ampWHALE should have been sent to the fee collector
                assert_eq!(funds, Uint128::zero());
            },
        )
        .query_flow(incentive_addr.clone().into_inner(), 1u64, |result| {
            let flow_response = result.unwrap();
            assert_eq!(
                flow_response.unwrap().flow,
                Some(Flow {
                    flow_id: 1,
                    flow_creator: carol.clone(),
                    flow_asset: Asset {
                        info: AssetInfo::NativeToken {
                            denom: "ampWHALE".to_string()
                        },
                        amount: Uint128::new(1_000u128),
                    },
                    claimed_amount: Uint128::zero(),
                    curve: Curve::Linear,
                    start_timestamp: app_time.clone().seconds(),
                    end_timestamp: app_time.clone().seconds(),
                    start_epoch: 1u64,
                    end_epoch: 10u64,
                    emissions_per_epoch: Default::default(),
                    emitted_tokens: Default::default(),
                })
            );
        })
        .query_flow(incentive_addr.clone().into_inner(), 5u64, |result| {
            // this should not work as there is no flow with id 5
            let flow_response = result.unwrap();
            assert_eq!(flow_response, None);
        })
        .query_funds(
            carol.clone(),
            AssetInfo::NativeToken {
                denom: "uwhale".clone().to_string(),
            },
            |result| {
                *carol_original_uwhale_funds.borrow_mut() = result;
            },
        )
        // create another incentive overpaying the fee, and check if the excees went back to carol
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "ampWHALE".clone().to_string(),
                },
                amount: Uint128::new(1_000u128),
            },
            &vec![
                coin(50_000u128, "uwhale".to_string()),
                coin(1_000u128, "ampWHALE".to_string()),
            ],
            |result| {
                // this should succeed as we sent enough funds to cover for fee + MIN_FLOW_AMOUNT
                result.unwrap();
            },
        )
        .query_funds(
            carol.clone(),
            AssetInfo::NativeToken {
                denom: "uwhale".clone().to_string(),
            },
            |result| {
                // the current balance should be the original minus the fee only, which is 1_000uwhale
                assert_eq!(
                    result,
                    carol_original_uwhale_funds.clone().into_inner() - Uint128::new(1_000u128)
                );
            },
        );
}

#[test]
fn open_flow_with_fee_native_token_and_flow_cw20_token() {
    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uwhale".to_string()),
        coin(1_000_000_000u128, "ampWHALE".to_string()),
    ]);
    let alice = suite.creator();
    let carol = suite.senders[2].clone();

    suite.instantiate_default_native_fee().create_lp_tokens();

    let mut fee_collector_addr = RefCell::new(Addr::unchecked(""));

    let lp_address_1 = AssetInfo::Token {
        contract_addr: suite.cw20_tokens.first().unwrap().to_string(),
    };

    let cw20_incentive = AssetInfo::Token {
        contract_addr: suite.cw20_tokens.last().unwrap().to_string(),
    };

    let cw20_incentive_address = suite.cw20_tokens.last().unwrap().clone();

    let mut incentive_addr = RefCell::new(Addr::unchecked(""));

    suite
        .create_incentive(alice.clone(), lp_address_1.clone(), |result| {
            result.unwrap();
        })
        .query_incentive(lp_address_1.clone(), |result| {
            let incentive = result.unwrap();
            assert!(incentive.is_some());
            *incentive_addr.borrow_mut() = incentive.unwrap();
        });

    let mut current_epoch = RefCell::new(0u64);
    suite.query_current_epoch(|result| {
        *current_epoch.borrow_mut() = result.unwrap().epoch.id.u64();
    });

    // open incentive flow
    let app_time = suite.get_time();
    suite
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: cw20_incentive.clone(),
                amount: Uint128::new(500u128),
            },
            &vec![coin(1_000u128, "uwhale".to_string())],
            |result| {
                // this should fail as MIN_FLOW_AMOUNT is not met
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::EmptyFlow { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::EmptyFlow"),
                }
            },
        )
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: cw20_incentive.clone(),
                amount: Uint128::new(1_000u128),
            },
            &vec![coin(1_000u128, "uwhale".to_string())],
            |result| {
                // this should fail as the flow asset was not sent, i.e. Allowance was not increased
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::FlowAssetNotSent { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::FlowAssetNotSent"),
                }
            },
        )
        .increase_allowance(
            carol.clone(),
            cw20_incentive_address.clone(),
            Uint128::new(1_000u128),
            incentive_addr.clone().into_inner(),
        )
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: cw20_incentive.clone(),
                amount: Uint128::new(1_000u128),
            },
            &vec![coin(1_000u128, "uwhale".to_string())],
            |result| {
                // this should succeed as the allowance was increased
                result.unwrap();
            },
        )
        .query_funds(
            incentive_addr.clone().into_inner(),
            cw20_incentive.clone(),
            |funds| {
                // funds on the incentive contract
                assert_eq!(funds, Uint128::new(1_000u128));
            },
        )
        .query_funds(
            incentive_addr.clone().into_inner(),
            AssetInfo::NativeToken {
                denom: "uwhale".to_string(),
            },
            |funds| {
                // no uwhale should have been sent to the incentive contract
                assert_eq!(funds, Uint128::zero());
            },
        )
        .query_incentive_factory_config(|result| {
            *fee_collector_addr.borrow_mut() = result.unwrap().fee_collector_addr;
        })
        .query_funds(
            fee_collector_addr.clone().into_inner(),
            AssetInfo::NativeToken {
                denom: "uwhale".to_string(),
            },
            |funds| {
                // funds on the fee collector
                assert_eq!(funds, Uint128::new(1_000u128));
            },
        )
        .query_funds(
            fee_collector_addr.clone().into_inner(),
            cw20_incentive.clone(),
            |funds| {
                // no cw20_incentive amount should have been sent to the fee collector
                assert_eq!(funds, Uint128::zero());
            },
        )
        .query_flow(incentive_addr.clone().into_inner(), 1u64, |result| {
            let flow_response = result.unwrap();
            assert_eq!(
                flow_response.unwrap().flow,
                Some(Flow {
                    flow_id: 1,
                    flow_creator: carol.clone(),
                    flow_asset: Asset {
                        info: cw20_incentive.clone(),
                        amount: Uint128::new(1_000u128),
                    },
                    claimed_amount: Uint128::zero(),
                    curve: Curve::Linear,
                    start_timestamp: app_time.clone().seconds(),
                    end_timestamp: app_time.clone().seconds(),
                    start_epoch: 1u64,
                    end_epoch: 10u64,
                    emissions_per_epoch: Default::default(),
                    emitted_tokens: Default::default(),
                })
            );
        });
}

#[test]
fn open_flow_with_fee_cw20_token_and_flow_same_cw20_token() {
    let mut suite =
        TestingSuite::default_with_balances(vec![coin(1_000_000_000u128, "uwhale".to_string())]);
    let alice = suite.creator();
    let carol = suite.senders[2].clone();

    suite.instantiate_default_cw20_fee().create_lp_tokens();

    let mut fee_collector_addr = RefCell::new(Addr::unchecked(""));

    let lp_address_last = AssetInfo::Token {
        contract_addr: suite.cw20_tokens.last().unwrap().to_string(),
    };

    let cw20_asset = AssetInfo::Token {
        contract_addr: suite.cw20_tokens.first().unwrap().to_string(),
    };

    let cw20_asset_addr = suite.cw20_tokens.first().unwrap().clone();

    let mut incentive_addr = RefCell::new(Addr::unchecked(""));

    suite
        .create_incentive(alice.clone(), lp_address_last.clone(), |result| {
            result.unwrap();
        })
        .query_incentive(lp_address_last.clone(), |result| {
            let incentive = result.unwrap();
            assert!(incentive.is_some());

            println!("incentive: {:?}", incentive);
            *incentive_addr.borrow_mut() = incentive.unwrap();
        });

    let mut current_epoch = RefCell::new(0u64);
    suite.query_current_epoch(|result| {
        *current_epoch.borrow_mut() = result.unwrap().epoch.id.u64();
    });

    // open incentive flow
    let app_time = suite.get_time();

    suite
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: cw20_asset.clone(),
                amount: Uint128::new(500u128),
            },
            &vec![],
            |result| {
                // this should fail as not enough funds were sent
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::EmptyFlow { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::EmptyFlow"),
                }
            },
        )
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: cw20_asset.clone(),
                amount: Uint128::new(1_000u128),
            },
            &vec![],
            |result| {
                // this should fail as not enough funds were sent to cover for fee
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::EmptyFlowAfterFee { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::EmptyFlowAfterFee"),
                }
            },
        )
        // let's increase the allowance but not enough to cover for the fees and MIN_FLOW_AMOUNT
        .increase_allowance(
            carol.clone(),
            cw20_asset_addr.clone(),
            Uint128::new(1_500u128),
            incentive_addr.clone().into_inner(),
        )
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: cw20_asset.clone(),
                amount: Uint128::new(1_500u128),
            },
            &vec![],
            |result| {
                // this should fail as not enough funds were sent to cover for fee and MIN_FLOW_AMOUNT
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::EmptyFlowAfterFee { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::EmptyFlowAfterFee"),
                }
            },
        )
        .increase_allowance(
            carol.clone(),
            cw20_asset_addr.clone(),
            Uint128::new(2_000u128),
            incentive_addr.clone().into_inner(),
        )
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: cw20_asset.clone(),
                amount: Uint128::new(2_000u128),
            },
            &vec![],
            |result| {
                // this should succeed as enough funds were sent to cover for fee and MIN_FLOW_AMOUNT
                result.unwrap();
            },
        )
        .query_funds(
            incentive_addr.clone().into_inner(),
            cw20_asset.clone(),
            |funds| {
                // funds on the incentive contract
                println!("funds on the incentive contract: {}", funds);
                assert_eq!(funds, Uint128::new(1_000u128));
            },
        )
        .query_incentive_factory_config(|result| {
            *fee_collector_addr.borrow_mut() = result.unwrap().fee_collector_addr;
        })
        .query_funds(
            fee_collector_addr.clone().into_inner(),
            cw20_asset.clone(),
            |funds| {
                // funds on the fee collector
                println!("funds on the fee collector: {}", funds);
                assert_eq!(funds, Uint128::new(1_000u128));
            },
        )
        .query_flow(incentive_addr.clone().into_inner(), 1u64, |result| {
            let flow_response = result.unwrap();
            assert_eq!(
                flow_response.unwrap().flow,
                Some(Flow {
                    flow_id: 1,
                    flow_creator: carol.clone(),
                    flow_asset: Asset {
                        info: cw20_asset.clone(),
                        amount: Uint128::new(1_000u128),
                    },
                    claimed_amount: Uint128::zero(),
                    curve: Curve::Linear,
                    start_timestamp: app_time.clone().seconds(),
                    end_timestamp: app_time.clone().seconds(),
                    start_epoch: 1u64,
                    end_epoch: 10u64,
                    emissions_per_epoch: Default::default(),
                    emitted_tokens: Default::default(),
                })
            );
        });
}

#[test]
fn open_flow_with_fee_cw20_token_and_flow_different_cw20_token() {
    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uwhale".to_string()),
        coin(1_000_000_000u128, "ampWHALE".to_string()),
    ]);
    let alice = suite.creator();
    let carol = suite.senders[2].clone();

    suite.instantiate_default_cw20_fee().create_lp_tokens();

    let mut fee_collector_addr = RefCell::new(Addr::unchecked(""));

    let cw20_fee_asset = AssetInfo::Token {
        contract_addr: suite.cw20_tokens.first().unwrap().to_string(),
    };
    let cw20_fee_asset_addr = suite.cw20_tokens.first().unwrap().clone();

    let cw20_asset = AssetInfo::Token {
        contract_addr: suite.cw20_tokens.last().unwrap().to_string(),
    };
    let cw20_asset_addr = suite.cw20_tokens.last().unwrap().clone();

    let mut incentive_addr = RefCell::new(Addr::unchecked(""));

    suite
        .create_incentive(alice.clone(), cw20_fee_asset.clone(), |result| {
            result.unwrap();
        })
        .query_incentive(cw20_fee_asset.clone(), |result| {
            let incentive = result.unwrap();
            assert!(incentive.is_some());
            *incentive_addr.borrow_mut() = incentive.unwrap();
        });

    let mut current_epoch = RefCell::new(0u64);
    suite.query_current_epoch(|result| {
        *current_epoch.borrow_mut() = result.unwrap().epoch.id.u64();
    });

    // open incentive flow
    let app_time = suite.get_time();
    suite
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: cw20_asset.clone(),
                amount: Uint128::new(500u128),
            },
            &vec![],
            |result| {
                // this should fail as not enough funds were sent
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::EmptyFlow { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::EmptyFlow"),
                }
            },
        )
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: cw20_asset.clone(),
                amount: Uint128::new(1_000u128),
            },
            &vec![],
            |result| {
                // this should fail as the asset to pay for the fee was not transferred
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::FlowFeeNotPaid { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::FlowFeeNotPaid"),
                }
            },
        )
        // incerase allowance for the flow asset, but not enough to cover the MIN_FLOW_AMOUNT
        .increase_allowance(
            carol.clone(),
            cw20_asset_addr.clone(),
            Uint128::new(500u128),
            incentive_addr.clone().into_inner(),
        )
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: cw20_asset.clone(),
                amount: Uint128::new(1_000u128),
            },
            &vec![],
            |result| {
                // this should fail as not enough funds were sent to cover for fee
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::FlowFeeNotPaid { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::FlowFeeNotPaid"),
                }
            },
        )
        // increase allowance for the fee asset
        .increase_allowance(
            carol.clone(),
            cw20_fee_asset_addr.clone(),
            Uint128::new(1_000u128),
            incentive_addr.clone().into_inner(),
        )
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: cw20_asset.clone(),
                amount: Uint128::new(1_000u128),
            },
            &vec![],
            |result| {
                // this should fail as not enough funds were sent to cover the flow asset
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::FlowAssetNotSent { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::FlowAssetNotSent"),
                }
            },
        )
        // increase allowance for the flow asset
        .increase_allowance(
            carol.clone(),
            cw20_asset_addr.clone(),
            Uint128::new(1_000u128),
            incentive_addr.clone().into_inner(),
        )
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: cw20_asset.clone(),
                amount: Uint128::new(1_000u128),
            },
            &vec![],
            |result| {
                // this should succeed as both the fee was paid in full and the flow asset amount
                // matches the one sent to the contract
                result.unwrap();
            },
        )
        .query_funds(
            incentive_addr.clone().into_inner(),
            cw20_asset.clone(),
            |funds| {
                // funds on the incentive contract
                assert_eq!(funds, Uint128::new(1_000u128));
            },
        )
        .query_funds(
            incentive_addr.clone().into_inner(),
            cw20_fee_asset.clone(),
            |funds| {
                // no cw20_fee_asset should have been sent to the incentive contract
                assert_eq!(funds, Uint128::zero());
            },
        )
        .query_incentive_factory_config(|result| {
            *fee_collector_addr.borrow_mut() = result.unwrap().fee_collector_addr;
        })
        .query_funds(
            fee_collector_addr.clone().into_inner(),
            cw20_asset.clone(),
            |funds| {
                // no flow assets on the fee collector
                assert_eq!(funds, Uint128::zero());
            },
        )
        .query_funds(
            fee_collector_addr.clone().into_inner(),
            cw20_fee_asset.clone(),
            |funds| {
                // cw20_fee_asset funds on the fee collector
                assert_eq!(funds, Uint128::new(1_000u128));
            },
        )
        .query_flow(incentive_addr.clone().into_inner(), 1u64, |result| {
            let flow_response = result.unwrap();
            assert_eq!(
                flow_response.unwrap().flow,
                Some(Flow {
                    flow_id: 1,
                    flow_creator: carol.clone(),
                    flow_asset: Asset {
                        info: cw20_asset.clone(),
                        amount: Uint128::new(1_000u128),
                    },
                    claimed_amount: Uint128::zero(),
                    curve: Curve::Linear,
                    start_timestamp: app_time.clone().seconds(),
                    end_timestamp: app_time.clone().seconds(),
                    start_epoch: 1u64,
                    end_epoch: 10u64,
                    emissions_per_epoch: Default::default(),
                    emitted_tokens: Default::default(),
                })
            );
        })
        .query_flow(incentive_addr.clone().into_inner(), 5u64, |result| {
            // this should not work as there is no flow with id 5
            let flow_response = result.unwrap();
            assert_eq!(flow_response, None);
        });
}

#[test]
fn open_flow_with_fee_cw20_token_and_flow_native_token() {
    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uwhale".to_string()),
        coin(1_000_000_000u128, "usdc".to_string()),
    ]);
    let alice = suite.creator();
    let carol = suite.senders[2].clone();

    suite.instantiate_default_cw20_fee().create_lp_tokens();

    let mut fee_collector_addr = RefCell::new(Addr::unchecked(""));

    let cw20_fee_asset = AssetInfo::Token {
        contract_addr: suite.cw20_tokens.first().unwrap().to_string(),
    };
    let cw20_fee_asset_addr = suite.cw20_tokens.first().unwrap().clone();

    let mut incentive_addr = RefCell::new(Addr::unchecked(""));

    suite
        .create_incentive(alice.clone(), cw20_fee_asset.clone(), |result| {
            result.unwrap();
        })
        .query_incentive(cw20_fee_asset.clone(), |result| {
            let incentive = result.unwrap();
            assert!(incentive.is_some());
            *incentive_addr.borrow_mut() = incentive.unwrap();
        });

    let mut carol_original_usdc_funds = RefCell::new(Uint128::zero());

    let mut current_epoch = RefCell::new(0u64);
    suite.query_current_epoch(|result| {
        *current_epoch.borrow_mut() = result.unwrap().epoch.id.u64();
    });

    // open incentive flow
    let app_time = suite.get_time();
    suite
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "usdc".to_string(),
                },
                amount: Uint128::new(500u128),
            },
            &vec![],
            |result| {
                // this should fail as not enough funds were sent
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::EmptyFlow { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::EmptyFlow"),
                }
            },
        )
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "usdc".to_string(),
                },
                amount: Uint128::new(1_000u128),
            },
            &vec![],
            |result| {
                // this should fail as the asset to pay for the fee was not transferred
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::FlowFeeNotPaid { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::FlowFeeNotPaid"),
                }
            },
        )
        // incerase allowance for the fee asset, but not enough
        .increase_allowance(
            carol.clone(),
            cw20_fee_asset_addr.clone(),
            Uint128::new(999u128),
            incentive_addr.clone().into_inner(),
        )
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "usdc".to_string(),
                },
                amount: Uint128::new(1_000u128),
            },
            &vec![],
            |result| {
                // this should fail as not enough funds were sent to cover for fee
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                println!("-----");
                match err {
                    ContractError::FlowFeeNotPaid { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::FlowFeeNotPaid"),
                }
            },
        )
        // incerase allowance for the fee asset, enough to cover the fee
        .increase_allowance(
            carol.clone(),
            cw20_fee_asset_addr.clone(),
            Uint128::new(1u128),
            incentive_addr.clone().into_inner(),
        )
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "usdc".to_string(),
                },
                amount: Uint128::new(1_000u128),
            },
            &vec![],
            |result| {
                // this should fail as the flow asset was not sent to the contract
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::FlowAssetNotSent { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::FlowAssetNotSent"),
                }
            },
        )
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "usdc".to_string(),
                },
                amount: Uint128::new(1_000u128),
            },
            &vec![coin(900u128, "usdc".to_string())],
            |result| {
                // this should fail as the flow asset was not sent to the contract
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::FlowAssetNotSent { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::FlowAssetNotSent"),
                }
            },
        )
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "usdc".to_string(),
                },
                amount: Uint128::new(1_000u128),
            },
            &vec![coin(1_000u128, "usdc".to_string())],
            |result| {
                // this should succeed as the flow asset was sent to the contract
                result.unwrap();
            },
        )
        .query_funds(
            incentive_addr.clone().into_inner(),
            AssetInfo::NativeToken {
                denom: "usdc".to_string(),
            },
            |funds| {
                // funds on the incentive contract
                assert_eq!(funds, Uint128::new(1_000u128));
            },
        )
        .query_funds(
            incentive_addr.clone().into_inner(),
            cw20_fee_asset.clone(),
            |funds| {
                // no cw20_fee_asset should have been sent to the incentive contract
                assert_eq!(funds, Uint128::zero());
            },
        )
        .query_incentive_factory_config(|result| {
            *fee_collector_addr.borrow_mut() = result.unwrap().fee_collector_addr;
        })
        .query_funds(
            fee_collector_addr.clone().into_inner(),
            AssetInfo::NativeToken {
                denom: "usdc".to_string(),
            },
            |funds| {
                // no flow assets on the fee collector
                assert_eq!(funds, Uint128::zero());
            },
        )
        .query_funds(
            fee_collector_addr.clone().into_inner(),
            cw20_fee_asset.clone(),
            |funds| {
                // cw20_fee_asset funds on the fee collector
                assert_eq!(funds, Uint128::new(1_000u128));
            },
        )
        .query_flow(incentive_addr.clone().into_inner(), 1u64, |result| {
            let flow_response = result.unwrap();
            assert_eq!(
                flow_response.unwrap().flow,
                Some(Flow {
                    flow_id: 1,
                    flow_creator: carol.clone(),
                    flow_asset: Asset {
                        info: AssetInfo::NativeToken {
                            denom: "usdc".to_string(),
                        },
                        amount: Uint128::new(1_000u128),
                    },
                    claimed_amount: Uint128::zero(),
                    curve: Curve::Linear,
                    start_timestamp: app_time.clone().seconds(),
                    end_timestamp: app_time.clone().seconds(),
                    start_epoch: 1u64,
                    end_epoch: 10u64,
                    emissions_per_epoch: Default::default(),
                    emitted_tokens: Default::default(),
                })
            );
        })
        .query_flow(incentive_addr.clone().into_inner(), 5u64, |result| {
            // this should not work as there is no flow with id 5
            let flow_response = result.unwrap();
            assert_eq!(flow_response, None);
        });
}

#[test]
fn close_native_token_flows() {
    let mut suite =
        TestingSuite::default_with_balances(vec![coin(1_000_000_000u128, "uwhale".to_string())]);
    let alice = suite.creator();
    let bob = suite.senders[1].clone();
    let carol = suite.senders[2].clone();

    suite.instantiate_default_native_fee().create_lp_tokens();

    let mut alice_funds = RefCell::new(Uint128::zero());
    let mut carol_funds = RefCell::new(Uint128::zero());

    let lp_address_1 = AssetInfo::Token {
        contract_addr: suite.cw20_tokens.first().unwrap().to_string(),
    };

    let mut incentive_addr = RefCell::new(Addr::unchecked(""));

    suite
        .create_incentive(alice.clone(), lp_address_1.clone(), |result| {
            result.unwrap();
        })
        .query_incentive(lp_address_1.clone(), |result| {
            let incentive = result.unwrap();
            assert!(incentive.is_some());
            *incentive_addr.borrow_mut() = incentive.unwrap();
        });

    let mut current_epoch = RefCell::new(0u64);
    suite.query_current_epoch(|result| {
        *current_epoch.borrow_mut() = result.unwrap().epoch.id.u64();
    });

    // open incentive flow
    let app_time = suite.get_time();

    suite
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uwhale".clone().to_string(),
                },
                amount: Uint128::new(2_000u128),
            },
            &vec![coin(2_000u128, "uwhale".to_string())],
            |result| {
                result.unwrap();
            },
        )
        .open_incentive_flow(
            alice.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uwhale".clone().to_string(),
                },
                amount: Uint128::new(11_000u128),
            },
            &vec![coin(11_000u128, "uwhale".to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_flows(incentive_addr.clone().into_inner(), |result| {
            let flows = result.unwrap();

            assert_eq!(flows.len(), 2usize);
            assert_eq!(
                flows.first().unwrap(),
                &Flow {
                    flow_id: 1,
                    flow_creator: carol.clone(),
                    flow_asset: Asset {
                        info: AssetInfo::NativeToken {
                            denom: "uwhale".to_string(),
                        },
                        amount: Uint128::new(1_000u128),
                    },
                    claimed_amount: Uint128::zero(),
                    curve: Curve::Linear,
                    start_timestamp: app_time.clone().seconds(),
                    end_timestamp: app_time.clone().seconds(),
                    start_epoch: 1u64,
                    end_epoch: 10u64,
                    emissions_per_epoch: Default::default(),
                    emitted_tokens: Default::default(),
                }
            );
            assert_eq!(
                flows.last().unwrap(),
                &Flow {
                    flow_id: 2,
                    flow_creator: alice.clone(),
                    flow_asset: Asset {
                        info: AssetInfo::NativeToken {
                            denom: "uwhale".to_string(),
                        },
                        amount: Uint128::new(10_000u128),
                    },
                    claimed_amount: Uint128::zero(),
                    curve: Curve::Linear,
                    start_timestamp: app_time.clone().seconds(),
                    end_timestamp: app_time.clone().seconds(),
                    start_epoch: 1u64,
                    end_epoch: 10u64,
                    emissions_per_epoch: Default::default(),
                    emitted_tokens: Default::default(),
                }
            );
        })
        .close_incentive_flow(
            bob.clone(),
            incentive_addr.clone().into_inner(),
            1u64,
            |result| {
                // this should error because bob didn't open the flow, nor he is the owner of the incentive
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::UnauthorizedFlowClose { .. } => {}
                    _ => panic!(
                        "Wrong error type, should return ContractError::UnauthorizedFlowClose"
                    ),
                }
            },
        )
        .close_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            2u64,
            |result| {
                // this should error because carol didn't open the flow, nor he is the owner of the incentive
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::UnauthorizedFlowClose { .. } => {}
                    _ => panic!(
                        "Wrong error type, should return ContractError::UnauthorizedFlowClose"
                    ),
                }
            },
        )
        .query_funds(
            alice.clone(),
            AssetInfo::NativeToken {
                denom: "uwhale".to_string(),
            },
            |funds| {
                *alice_funds.borrow_mut() = funds;
            },
        )
        // alice closes her flow
        .close_incentive_flow(
            alice.clone(),
            incentive_addr.clone().into_inner(),
            2u64,
            |result| {
                // this should be fine because carol opened the flow
                result.unwrap();
            },
        )
        .query_flows(incentive_addr.clone().into_inner(), |result| {
            let flows = result.unwrap();

            assert_eq!(flows.len(), 1usize);
            assert_eq!(
                flows.last().unwrap(),
                &Flow {
                    flow_id: 1,
                    flow_creator: carol.clone(),
                    flow_asset: Asset {
                        info: AssetInfo::NativeToken {
                            denom: "uwhale".to_string(),
                        },
                        amount: Uint128::new(1_000u128),
                    },
                    claimed_amount: Uint128::zero(),
                    curve: Curve::Linear,
                    start_timestamp: app_time.clone().seconds(),
                    end_timestamp: app_time.clone().seconds(),
                    start_epoch: 1u64,
                    end_epoch: 10u64,
                    emissions_per_epoch: Default::default(),
                    emitted_tokens: Default::default(),
                }
            );
        })
        .query_funds(
            alice.clone(),
            AssetInfo::NativeToken {
                denom: "uwhale".to_string(),
            },
            |funds| {
                // since nothing from the flow was claimed, it means 10_000u128uwhale was returned to alice
                assert_eq!(
                    funds - alice_funds.clone().into_inner(),
                    Uint128::new(10_000u128)
                );
                *alice_funds.borrow_mut() = funds;
            },
        )
        .query_funds(
            carol.clone(),
            AssetInfo::NativeToken {
                denom: "uwhale".to_string(),
            },
            |funds| {
                *carol_funds.borrow_mut() = funds;
            },
        )
        // alice closes carols flow. She can do it since she is the owner of the flow
        .close_incentive_flow(
            alice.clone(),
            incentive_addr.clone().into_inner(),
            1u64,
            |result| {
                result.unwrap();
            },
        )
        .query_funds(
            carol.clone(),
            AssetInfo::NativeToken {
                denom: "uwhale".to_string(),
            },
            |funds| {
                // since nothing from the flow was claimed, it means 1_000u128uwhale was returned to carol
                assert_eq!(
                    funds - carol_funds.clone().into_inner(),
                    Uint128::new(1_000u128)
                );
            },
        )
        .query_flows(incentive_addr.clone().into_inner(), |result| {
            let flows = result.unwrap();
            assert!(flows.is_empty());
        })
        // try closing a flow that doesn't exist
        .close_incentive_flow(
            bob.clone(),
            incentive_addr.clone().into_inner(),
            3u64,
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::NonExistentFlow { invalid_id } => {
                        assert_eq!(invalid_id, 3u64)
                    }
                    _ => panic!("Wrong error type, should return ContractError::NonExistentFlow"),
                }
            },
        )
        .open_incentive_flow(
            alice.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uwhale".clone().to_string(),
                },
                amount: Uint128::new(5_000u128),
            },
            &vec![coin(5_000u128, "uwhale".to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_flows(incentive_addr.clone().into_inner(), |result| {
            let flows = result.unwrap();

            assert_eq!(flows.len(), 1usize);
            assert_eq!(
                flows.last().unwrap(),
                &Flow {
                    flow_id: 3,
                    flow_creator: alice.clone(),
                    flow_asset: Asset {
                        info: AssetInfo::NativeToken {
                            denom: "uwhale".to_string(),
                        },
                        amount: Uint128::new(4_000u128),
                    },
                    claimed_amount: Uint128::zero(),
                    curve: Curve::Linear,
                    start_timestamp: app_time.clone().seconds(),
                    end_timestamp: app_time.clone().seconds(),
                    start_epoch: 1u64,
                    end_epoch: 10u64,
                    emissions_per_epoch: Default::default(),
                    emitted_tokens: Default::default(),
                }
            );
        });
}

#[test]
fn close_cw20_token_flows() {
    let mut suite =
        TestingSuite::default_with_balances(vec![coin(1_000_000_000u128, "uwhale".to_string())]);
    let alice = suite.creator();
    let bob = suite.senders[1].clone();
    let carol = suite.senders[2].clone();

    suite.instantiate_default_native_fee().create_lp_tokens();

    let mut alice_funds = RefCell::new(Uint128::zero());
    let mut carol_funds = RefCell::new(Uint128::zero());

    let lp_address_1 = AssetInfo::Token {
        contract_addr: suite.cw20_tokens.first().unwrap().to_string(),
    };

    let cw20_asset = AssetInfo::Token {
        contract_addr: suite.cw20_tokens.last().unwrap().to_string(),
    };
    let cw20_asset_addr = suite.cw20_tokens.last().unwrap().clone();

    let mut incentive_addr = RefCell::new(Addr::unchecked(""));

    suite
        .create_incentive(alice.clone(), lp_address_1.clone(), |result| {
            result.unwrap();
        })
        .query_incentive(lp_address_1.clone(), |result| {
            let incentive = result.unwrap();
            assert!(incentive.is_some());
            *incentive_addr.borrow_mut() = incentive.unwrap();
        });

    let mut current_epoch = RefCell::new(0u64);
    suite.query_current_epoch(|result| {
        *current_epoch.borrow_mut() = result.unwrap().epoch.id.u64();
    });

    // open incentive flow
    let app_time = suite.get_time();

    suite
        .increase_allowance(
            carol.clone(),
            cw20_asset_addr.clone(),
            Uint128::new(1_000u128),
            incentive_addr.clone().into_inner(),
        )
        .open_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: cw20_asset.clone(),
                amount: Uint128::new(1_000u128),
            },
            &vec![coin(1_000u128, "uwhale".to_string())],
            |result| {
                result.unwrap();
            },
        )
        .increase_allowance(
            alice.clone(),
            cw20_asset_addr.clone(),
            Uint128::new(10_000u128),
            incentive_addr.clone().into_inner(),
        )
        .open_incentive_flow(
            alice.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: cw20_asset.clone(),
                amount: Uint128::new(10_000u128),
            },
            &vec![coin(1_000u128, "uwhale".to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_flows(incentive_addr.clone().into_inner(), |result| {
            let flows = result.unwrap();

            assert_eq!(flows.len(), 2usize);
            assert_eq!(
                flows.first().unwrap(),
                &Flow {
                    flow_id: 1,
                    flow_creator: carol.clone(),
                    flow_asset: Asset {
                        info: cw20_asset.clone(),
                        amount: Uint128::new(1_000u128),
                    },
                    claimed_amount: Uint128::zero(),
                    curve: Curve::Linear,
                    start_timestamp: app_time.clone().seconds(),
                    end_timestamp: app_time.clone().seconds(),
                    start_epoch: 1u64,
                    end_epoch: 10u64,
                    emissions_per_epoch: Default::default(),
                    emitted_tokens: Default::default(),
                }
            );
            assert_eq!(
                flows.last().unwrap(),
                &Flow {
                    flow_id: 2,
                    flow_creator: alice.clone(),
                    flow_asset: Asset {
                        info: cw20_asset.clone(),
                        amount: Uint128::new(10_000u128),
                    },
                    claimed_amount: Uint128::zero(),
                    curve: Curve::Linear,
                    start_timestamp: app_time.clone().seconds(),
                    end_timestamp: app_time.clone().seconds(),
                    start_epoch: 1u64,
                    end_epoch: 10u64,
                    emissions_per_epoch: Default::default(),
                    emitted_tokens: Default::default(),
                }
            );
        })
        .close_incentive_flow(
            bob.clone(),
            incentive_addr.clone().into_inner(),
            1u64,
            |result| {
                // this should error because bob didn't open the flow, nor he is the owner of the incentive
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::UnauthorizedFlowClose { .. } => {}
                    _ => panic!(
                        "Wrong error type, should return ContractError::UnauthorizedFlowClose"
                    ),
                }
            },
        )
        .close_incentive_flow(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            2u64,
            |result| {
                // this should error because carol didn't open the flow, nor he is the owner of the incentive
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::UnauthorizedFlowClose { .. } => {}
                    _ => panic!(
                        "Wrong error type, should return ContractError::UnauthorizedFlowClose"
                    ),
                }
            },
        )
        .query_funds(alice.clone(), cw20_asset.clone(), |funds| {
            *alice_funds.borrow_mut() = funds;
        })
        // alice closes her flow
        .close_incentive_flow(
            alice.clone(),
            incentive_addr.clone().into_inner(),
            2u64,
            |result| {
                // this should be fine because carol opened the flow
                result.unwrap();
            },
        )
        .query_flows(incentive_addr.clone().into_inner(), |result| {
            let flows = result.unwrap();

            assert_eq!(flows.len(), 1usize);
            assert_eq!(
                flows.last().unwrap(),
                &Flow {
                    flow_id: 1,
                    flow_creator: carol.clone(),
                    flow_asset: Asset {
                        info: cw20_asset.clone(),
                        amount: Uint128::new(1_000u128),
                    },
                    claimed_amount: Uint128::zero(),
                    curve: Curve::Linear,
                    start_timestamp: app_time.clone().seconds(),
                    end_timestamp: app_time.clone().seconds(),
                    start_epoch: 1u64,
                    end_epoch: 10u64,
                    emissions_per_epoch: Default::default(),
                    emitted_tokens: Default::default(),
                }
            );
        })
        .query_funds(alice.clone(), cw20_asset.clone(), |funds| {
            // since nothing from the flow was claimed, it means 10_000u128 cw20_asset was returned to alice
            assert_eq!(
                funds - alice_funds.clone().into_inner(),
                Uint128::new(10_000u128)
            );
            *alice_funds.borrow_mut() = funds;
        })
        .query_funds(carol.clone(), cw20_asset.clone(), |funds| {
            *carol_funds.borrow_mut() = funds;
        })
        // alice closes carols flow. She can do it since she is the owner of the flow
        .close_incentive_flow(
            alice.clone(),
            incentive_addr.clone().into_inner(),
            1u64,
            |result| {
                result.unwrap();
            },
        )
        .query_funds(carol.clone(), cw20_asset.clone(), |funds| {
            // since nothing from the flow was claimed, it means 1_000u128 cw20_asset was returned to carol
            assert_eq!(
                funds - carol_funds.clone().into_inner(),
                Uint128::new(1_000u128)
            );
        })
        .query_flows(incentive_addr.clone().into_inner(), |result| {
            let flows = result.unwrap();
            assert!(flows.is_empty());
        })
        // try closing a flow that doesn't exist
        .close_incentive_flow(
            bob.clone(),
            incentive_addr.clone().into_inner(),
            3u64,
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::NonExistentFlow { invalid_id } => {
                        assert_eq!(invalid_id, 3u64)
                    }
                    _ => panic!("Wrong error type, should return ContractError::NonExistentFlow"),
                }
            },
        )
        .increase_allowance(
            alice.clone(),
            cw20_asset_addr.clone(),
            Uint128::new(5_000u128),
            incentive_addr.clone().into_inner(),
        )
        .open_incentive_flow(
            alice.clone(),
            incentive_addr.clone().into_inner(),
            None,
            app_time.clone().plus_seconds(86400u64).seconds(),
            None,
            current_epoch.clone().into_inner() + 9,
            Curve::Linear,
            Asset {
                info: cw20_asset.clone(),
                amount: Uint128::new(5_000u128),
            },
            &vec![coin(1_000u128, "uwhale".to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_flows(incentive_addr.clone().into_inner(), |result| {
            let flows = result.unwrap();

            assert_eq!(flows.len(), 1usize);
            assert_eq!(
                flows.last().unwrap(),
                &Flow {
                    flow_id: 3,
                    flow_creator: alice.clone(),
                    flow_asset: Asset {
                        info: cw20_asset.clone(),
                        amount: Uint128::new(5_000u128),
                    },
                    claimed_amount: Uint128::zero(),
                    curve: Curve::Linear,
                    start_timestamp: app_time.clone().seconds(),
                    end_timestamp: app_time.clone().seconds(),
                    start_epoch: 1u64,
                    end_epoch: 10u64,
                    emissions_per_epoch: Default::default(),
                    emitted_tokens: Default::default(),
                }
            );
        });
}

#[test]
fn open_flow_positions_and_claim_native_token_incentive() {
    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uwhale".to_string()),
        coin(1_000_000_000u128, "usdc".to_string()),
        coin(1_000_000_000u128, "ampWHALE".to_string()),
    ]);
    let alice = suite.creator();
    let carol = suite.senders[2].clone();

    suite.instantiate_default_native_fee().create_lp_tokens();

    let incentive_asset = AssetInfo::NativeToken {
        denom: "ampWHALE".to_string(),
    };

    let mut incentive_addr = RefCell::new(Addr::unchecked(""));

    suite
        .create_incentive(alice.clone(), incentive_asset.clone(), |result| {
            result.unwrap();
        })
        .query_incentive(incentive_asset.clone(), |result| {
            let incentive = result.unwrap();
            assert!(incentive.is_some());
            *incentive_addr.borrow_mut() = incentive.unwrap();
        })
        .query_incentive_config(incentive_addr.clone().into_inner(), |result| {
            let config = result.unwrap();
            assert_eq!(config.lp_asset, incentive_asset.clone());
        });

    let broken_open_position = incentive::OpenPosition {
        amount: Uint128::zero(),
        unbonding_duration: 0u64,
    };
    suite.open_incentive_position(
        carol.clone(),
        incentive_addr.clone().into_inner(),
        broken_open_position.amount,
        broken_open_position.unbonding_duration,
        None,
        vec![],
        |result| {
            // this should fail since the unbonding duration cannot be less than the minimum configured
            // on the incentive factory
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();

            match err {
                ContractError::InvalidUnbondingDuration { .. } => {}
                _ => panic!(
                    "Wrong error type, should return ContractError::InvalidUnbondingDuration"
                ),
            }
        },
    );

    let broken_open_position = incentive::OpenPosition {
        amount: Uint128::zero(),
        unbonding_duration: 259300u64,
    };
    suite.open_incentive_position(
        carol.clone(),
        incentive_addr.clone().into_inner(),
        broken_open_position.amount,
        broken_open_position.unbonding_duration,
        None,
        vec![],
        |result| {
            // this should fail since the unbonding duration cannot be more than the maximum configured
            // on the incentive factory
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();

            match err {
                ContractError::InvalidUnbondingDuration { .. } => {}
                _ => panic!(
                    "Wrong error type, should return ContractError::InvalidUnbondingDuration"
                ),
            }
        },
    );

    let broken_open_position = incentive::OpenPosition {
        amount: Uint128::zero(),
        unbonding_duration: 86400u64,
    };
    suite.open_incentive_position(
        carol.clone(),
        incentive_addr.clone().into_inner(),
        broken_open_position.amount,
        broken_open_position.unbonding_duration,
        None,
        vec![],
        |result| {
            // this should fail since the amount is zero
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();

            match err {
                ContractError::PaymentError { .. } => {}
                _ => panic!("Wrong error type, should return ContractError::PaymentError"),
            }
        },
    );

    let open_position = incentive::OpenPosition {
        amount: Uint128::new(1_000u128),
        unbonding_duration: 86400u64,
    };
    suite
        .open_incentive_position(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            open_position.amount,
            open_position.unbonding_duration,
            None,
            vec![coin(1_000u128, "uwhale".to_string())],
            |result| {
                // this should fail since ampWHALE is missing
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::PaymentError { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::PaymentError"),
                }
            },
        )
        .open_incentive_position(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            open_position.amount,
            open_position.unbonding_duration,
            None,
            vec![
                coin(1_000u128, "uwhale".to_string()),
                coin(1_000u128, "usdc".to_string()),
            ],
            |result| {
                // this should fail since multiple denoms were sent
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::PaymentError { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::PaymentError"),
                }
            },
        )
        .open_incentive_position(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            open_position.amount,
            open_position.unbonding_duration,
            None,
            vec![coin(2_000u128, "ampWHALE".to_string())],
            |result| {
                // this should fail since the right amount wasn't sent, i.e. 1000 ampWHALE
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::MissingPositionDepositNative { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::MissingPositionDepositNative"),
                }
            },
        )
        .open_incentive_position(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            open_position.amount,
            open_position.unbonding_duration,
            None,
            vec![coin(1_000u128, "ampWHALE".to_string())],
            |result| {
                result.unwrap();
            },
        );

    let open_position = incentive::OpenPosition {
        amount: Uint128::new(2_000u128),
        unbonding_duration: 86400u64,
    };

    suite
        .open_incentive_position(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            open_position.amount,
            open_position.unbonding_duration,
            None,
            vec![coin(2_000u128, "ampWHALE".to_string())],
            |result| {
                // this should fail because you can't open a position with the same unbonding period
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::DuplicatePosition { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::DuplicatePosition"),
                }
            },
        )
        .query_positions(
            incentive_addr.clone().into_inner(),
            carol.clone(),
            |result| {
                assert_eq!(
                    result.unwrap().positions.first().unwrap(),
                    &incentive::QueryPosition::OpenPosition {
                        amount: Uint128::new(1_000u128),
                        unbonding_duration: open_position.unbonding_duration,
                        weight: Uint128::new(1_000u128),
                    }
                );
            },
        );

    suite.query_rewards(
        incentive_addr.clone().into_inner(),
        carol.clone(),
        |result| {
            // the incentive doesn't have any flows, so rewards should be empty
            assert!(result.unwrap().rewards.is_empty());
            println!("---------------");
        },
    );

    let time = Timestamp::from_seconds(1684766796u64);
    suite.set_time(time);

    let mut current_epoch = RefCell::new(0u64);
    suite
        .create_epochs_on_fee_distributor(10, vec![incentive_addr.clone().into_inner()])
        .query_current_epoch(|result| {
            *current_epoch.borrow_mut() = result.unwrap().epoch.id.u64();
        });

    let mut carol_usdc_funds = RefCell::new(Uint128::zero());
    println!("CURRENT_EPOCH  -> {:?}", current_epoch);
    suite
        .open_incentive_flow(
            alice.clone(),
            incentive_addr.clone().into_inner(),
            None,
            time.plus_seconds(172800u64).seconds(), //2 days
            None,
            current_epoch.clone().into_inner() + 10,
            Curve::Linear,
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "usdc".to_string(),
                },
                amount: Uint128::new(1_000_000_000u128),
            },
            &vec![coin(1_000_000_000u128, "usdc"), coin(1_000u128, "uwhale")],
            |result| {
                result.unwrap();
            },
        )
        // move time a day forward, so given that the flow ends in a day, Carol should have 50%
        // of the rewards (as she owns 100% of the pool)
        .set_time(time.plus_seconds(86400u64))
        .create_epochs_on_fee_distributor(4, vec![incentive_addr.clone().into_inner()]) // epoch is 15 now, half way of the duration of the flow
        .query_rewards(
            incentive_addr.clone().into_inner(),
            carol.clone(),
            |result| {
                println!("result -> {:?}", result);

                assert_eq!(
                    result.unwrap().rewards,
                    vec![Asset {
                        info: AssetInfo::NativeToken {
                            denom: "usdc".to_string(),
                        },
                        amount: Uint128::new(500_000_000u128),
                    },]
                );
            },
        )
        .query_funds(
            carol.clone(),
            AssetInfo::NativeToken {
                denom: "usdc".to_string(),
            },
            |result| {
                *carol_usdc_funds.borrow_mut() = result;
            },
        )
        .claim(
            incentive_addr.clone().into_inner(),
            carol.clone(),
            |result| {
                result.unwrap();
            },
        )
        .query_funds(
            carol.clone(),
            AssetInfo::NativeToken {
                denom: "usdc".to_string(),
            },
            |result| {
                assert_eq!(
                    result,
                    carol_usdc_funds
                        .clone()
                        .into_inner()
                        .checked_add(Uint128::new(500_000_000u128))
                        .unwrap(),
                );
            },
        )
        .query_flow(incentive_addr.clone().into_inner(), 1u64, |result| {
            let flow_response = result.unwrap();
            assert_eq!(
                flow_response.unwrap().flow.unwrap().claimed_amount,
                Uint128::new(500_000_000u128)
            );
        });

    // move 3 more epochs, so carol should have 300 more to claim
    suite
        .set_time(time.plus_seconds(129600u64))
        .create_epochs_on_fee_distributor(3, vec![incentive_addr.clone().into_inner()])
        .query_rewards(
            incentive_addr.clone().into_inner(),
            carol.clone(),
            |result| {
                assert_eq!(
                    result.unwrap().rewards,
                    vec![Asset {
                        info: AssetInfo::NativeToken {
                            denom: "usdc".to_string(),
                        },
                        amount: Uint128::new(300_000_000u128),
                    },]
                );
            },
        )
        // move 2 more epochs, so carol should have an additional 200_000_000usdc to claim.
        .set_time(time.plus_seconds(172800u64))
        .create_epochs_on_fee_distributor(2, vec![incentive_addr.clone().into_inner()])
        .query_rewards(
            incentive_addr.clone().into_inner(),
            carol.clone(),
            |result| {
                assert_eq!(
                    result.unwrap().rewards,
                    vec![Asset {
                        info: AssetInfo::NativeToken {
                            denom: "usdc".to_string(),
                        },
                        amount: Uint128::new(500_000_000u128),
                    },]
                );
            },
        ) // go beyond the end time of the flow, create one more epoch
        .set_time(time.plus_seconds(190000u64))
        .create_epochs_on_fee_distributor(1, vec![incentive_addr.clone().into_inner()]);

    println!("----999994*****------");

    suite
        .query_rewards(
            incentive_addr.clone().into_inner(),
            carol.clone(),
            |result| {
                assert_eq!(
                    result.unwrap().rewards,
                    // this should still return the remaining that has not been claimed, which is 500_000_000usdc
                    vec![Asset {
                        info: AssetInfo::NativeToken {
                            denom: "usdc".to_string(),
                        },
                        amount: Uint128::new(500_000_000u128),
                    },]
                );
            },
        )
        .claim(
            incentive_addr.clone().into_inner(),
            carol.clone(),
            |result| {
                result.unwrap();
            },
        )
        .query_flow(incentive_addr.clone().into_inner(), 1u64, |result| {
            let flow_response = result.unwrap();
            assert_eq!(
                flow_response.unwrap().flow.unwrap().claimed_amount,
                Uint128::new(1_000_000_000u128)
            );
        })
        .query_funds(
            carol.clone(),
            AssetInfo::NativeToken {
                denom: "usdc".to_string(),
            },
            |result| {
                assert_eq!(
                    result,
                    carol_usdc_funds
                        .clone()
                        .into_inner()
                        .checked_add(Uint128::new(1_000_000_000u128))
                        .unwrap(),
                );
            },
        )
        .query_rewards(
            incentive_addr.clone().into_inner(),
            carol.clone(),
            |result| {
                // There's nothing left to claim
                let err = result.unwrap_err();
                match err {
                    StdError::GenericErr { msg } => {
                        assert_eq!(
                            msg.rsplit_once(": ").unwrap().1,
                            (ContractError::NothingToClaim {}).to_string()
                        );
                    }
                    _ => panic!("Unexpected error type"),
                }
            },
        );
}

#[test]
fn open_flow_positions_claim_cw20_token_incentive() {
    let mut suite =
        TestingSuite::default_with_balances(vec![coin(1_000_000_000u128, "uwhale".to_string())]);
    let alice = suite.creator();
    let carol = suite.senders[2].clone();

    suite.instantiate_default_native_fee().create_lp_tokens();

    let incentive_asset = AssetInfo::Token {
        contract_addr: suite.cw20_tokens.first().unwrap().to_string(),
    };

    let incentive_asset_addr = suite.cw20_tokens.first().unwrap().clone();

    let flow_asset = AssetInfo::Token {
        contract_addr: suite.cw20_tokens.last().unwrap().to_string(),
    };

    let flow_asset_addr = suite.cw20_tokens.last().unwrap().clone();

    let mut incentive_addr = RefCell::new(Addr::unchecked(""));

    suite
        .create_incentive(alice.clone(), incentive_asset.clone(), |result| {
            result.unwrap();
        })
        .query_incentive(incentive_asset.clone(), |result| {
            let incentive = result.unwrap();
            assert!(incentive.is_some());
            *incentive_addr.borrow_mut() = incentive.unwrap();
        })
        .query_incentive_config(incentive_addr.clone().into_inner(), |result| {
            let config = result.unwrap();
            assert_eq!(config.lp_asset, incentive_asset.clone());
        });

    let broken_open_position = incentive::OpenPosition {
        amount: Uint128::zero(),
        unbonding_duration: 0u64,
    };
    suite.open_incentive_position(
        carol.clone(),
        incentive_addr.clone().into_inner(),
        broken_open_position.amount,
        broken_open_position.unbonding_duration,
        None,
        vec![],
        |result| {
            // this should fail since the unbonding duration cannot be less than the minimum configured
            // on the incentive factory
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();

            match err {
                ContractError::InvalidUnbondingDuration { .. } => {}
                _ => panic!(
                    "Wrong error type, should return ContractError::InvalidUnbondingDuration"
                ),
            }
        },
    );

    let broken_open_position = incentive::OpenPosition {
        amount: Uint128::zero(),
        unbonding_duration: 259300u64,
    };
    suite.open_incentive_position(
        carol.clone(),
        incentive_addr.clone().into_inner(),
        broken_open_position.amount,
        broken_open_position.unbonding_duration,
        None,
        vec![],
        |result| {
            // this should fail since the unbonding duration cannot be more than the maximum configured
            // on the incentive factory
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();

            match err {
                ContractError::InvalidUnbondingDuration { .. } => {}
                _ => panic!(
                    "Wrong error type, should return ContractError::InvalidUnbondingDuration"
                ),
            }
        },
    );

    let broken_open_position = incentive::OpenPosition {
        amount: Uint128::zero(),
        unbonding_duration: 86400u64,
    };
    suite.open_incentive_position(
        carol.clone(),
        incentive_addr.clone().into_inner(),
        broken_open_position.amount,
        broken_open_position.unbonding_duration,
        None,
        vec![],
        |result| {
            // this should fail since the amount is zero
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();

            match err {
                ContractError::PaymentError { .. } => {}
                _ => panic!("Wrong error type, should return ContractError::PaymentError"),
            }
        },
    );

    let open_position = incentive::OpenPosition {
        amount: Uint128::new(1_000u128),
        unbonding_duration: 86400u64,
    };
    suite
        .open_incentive_position(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            open_position.amount,
            open_position.unbonding_duration,
            None,
            vec![coin(1_000u128, "uwhale".to_string())],
            |result| {
                // this should fail since ampWHALE is missing
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::MissingPositionDeposit { .. } => {}
                    _ => panic!(
                        "Wrong error type, should return ContractError::MissingPositionDeposit"
                    ),
                }
            },
        )
        .increase_allowance(
            carol.clone(),
            incentive_asset_addr.clone(),
            Uint128::new(1_000u128),
            incentive_addr.clone().into_inner(),
        )
        .open_incentive_position(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            open_position.amount,
            open_position.unbonding_duration,
            None,
            vec![coin(1_000u128, "uwhale".to_string())],
            |result| {
                // this should be fine as the allowance was increased to match the position amount
                result.unwrap();
            },
        );

    let open_position = incentive::OpenPosition {
        amount: Uint128::new(2_000u128),
        unbonding_duration: 86400u64,
    };

    suite
        .increase_allowance(
            carol.clone(),
            incentive_asset_addr.clone(),
            Uint128::new(2_000u128),
            incentive_addr.clone().into_inner(),
        )
        .open_incentive_position(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            open_position.amount,
            open_position.unbonding_duration,
            None,
            vec![coin(1_000u128, "uwhale".to_string())],
            |result| {
                // this should fail because you can't open a position with the same unbonding period
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::DuplicatePosition { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::DuplicatePosition"),
                }
            },
        )
        .query_positions(
            incentive_addr.clone().into_inner(),
            carol.clone(),
            |result| {
                assert_eq!(
                    result.unwrap().positions.first().unwrap(),
                    &incentive::QueryPosition::OpenPosition {
                        amount: Uint128::new(1_000u128),
                        unbonding_duration: open_position.unbonding_duration,
                        weight: Uint128::new(1_000u128),
                    }
                );
            },
        );

    suite.query_rewards(
        incentive_addr.clone().into_inner(),
        carol.clone(),
        |result| {
            // the incentive doesn't have any flows, so rewards should be empty
            assert!(result.unwrap().rewards.is_empty());
            println!("---------------");
        },
    );

    let time = Timestamp::from_seconds(1684766796u64);
    suite.set_time(time);

    let mut current_epoch = RefCell::new(0u64);
    suite
        .create_epochs_on_fee_distributor(10, vec![incentive_addr.clone().into_inner()])
        .query_current_epoch(|result| {
            *current_epoch.borrow_mut() = result.unwrap().epoch.id.u64();
        });

    let mut carol_cw20_funds = RefCell::new(Uint128::zero());

    suite
        .increase_allowance(
            alice.clone(),
            flow_asset_addr.clone(),
            Uint128::new(1_000_000_000u128),
            incentive_addr.clone().into_inner(),
        )
        .open_incentive_flow(
            alice.clone(),
            incentive_addr.clone().into_inner(),
            None,
            time.plus_seconds(172800u64).seconds(), //2 days
            None,
            current_epoch.clone().into_inner() + 10,
            Curve::Linear,
            Asset {
                info: flow_asset.clone(),
                amount: Uint128::new(1_000_000_000u128),
            },
            &vec![coin(1_000u128, "uwhale")],
            |result| {
                result.unwrap();
            },
        )
        // move time a day forward, so given that the flow ends in a day, Carol should have 50%
        // of the rewards (as she owns 100% of the pool)
        .set_time(time.plus_seconds(86400u64))
        .create_epochs_on_fee_distributor(4, vec![incentive_addr.clone().into_inner()]) // epoch is 15 now, half way of the duration of the flow
        .query_rewards(
            incentive_addr.clone().into_inner(),
            carol.clone(),
            |result| {
                assert_eq!(
                    result.unwrap().rewards,
                    vec![Asset {
                        info: flow_asset.clone(),
                        amount: Uint128::new(500_000_000u128),
                    },]
                );
            },
        )
        .query_funds(carol.clone(), flow_asset.clone(), |result| {
            *carol_cw20_funds.borrow_mut() = result;
        })
        .claim(
            incentive_addr.clone().into_inner(),
            carol.clone(),
            |result| {
                result.unwrap();
            },
        )
        .query_funds(carol.clone(), flow_asset.clone(), |result| {
            assert_eq!(
                result,
                carol_cw20_funds
                    .clone()
                    .into_inner()
                    .checked_add(Uint128::new(500_000_000u128))
                    .unwrap(),
            );
        })
        .query_flow(incentive_addr.clone().into_inner(), 1u64, |result| {
            let flow_response = result.unwrap();
            assert_eq!(
                flow_response.unwrap().flow.unwrap().claimed_amount,
                Uint128::new(500_000_000u128)
            );
        });

    // move 3 more epochs, so carol should have 300 more to claim
    suite
        .set_time(time.plus_seconds(129600u64))
        .create_epochs_on_fee_distributor(3, vec![incentive_addr.clone().into_inner()])
        .query_rewards(
            incentive_addr.clone().into_inner(),
            carol.clone(),
            |result| {
                assert_eq!(
                    result.unwrap().rewards,
                    vec![Asset {
                        info: flow_asset.clone(),
                        amount: Uint128::new(300_000_000u128),
                    },]
                );
            },
        )
        .query_flow(incentive_addr.clone().into_inner(), 1u64, |result| {
            let flow_response = result.unwrap();
            assert_eq!(
                flow_response.unwrap().flow.unwrap().claimed_amount,
                Uint128::new(500_000_000u128)
            );
        })
        // move 2 more epochs, so carol should have an additional 200_000_000usdc to claim.
        .set_time(time.plus_seconds(172800u64))
        .create_epochs_on_fee_distributor(2, vec![incentive_addr.clone().into_inner()])
        .query_rewards(
            incentive_addr.clone().into_inner(),
            carol.clone(),
            |result| {
                assert_eq!(
                    result.unwrap().rewards,
                    vec![Asset {
                        info: flow_asset.clone(),
                        amount: Uint128::new(500_000_000u128),
                    },]
                );
            },
        ) // go beyond the end time of the flow
        .set_time(time.plus_seconds(190000u64))
        .create_epochs_on_fee_distributor(1, vec![incentive_addr.clone().into_inner()])
        .query_rewards(
            incentive_addr.clone().into_inner(),
            carol.clone(),
            |result| {
                assert_eq!(
                    result.unwrap().rewards,
                    // this should still return the remaining that has not been claimed, which is 500_000_000usdc
                    vec![Asset {
                        info: flow_asset.clone(),
                        amount: Uint128::new(500_000_000u128),
                    },]
                );
            },
        )
        .claim(
            incentive_addr.clone().into_inner(),
            carol.clone(),
            |result| {
                result.unwrap();
            },
        )
        .query_flow(incentive_addr.clone().into_inner(), 1u64, |result| {
            let flow_response = result.unwrap();
            assert_eq!(
                flow_response.unwrap().flow.unwrap().claimed_amount,
                Uint128::new(1_000_000_000u128)
            );
        })
        .query_funds(carol.clone(), flow_asset.clone(), |result| {
            assert_eq!(
                result,
                carol_cw20_funds
                    .clone()
                    .into_inner()
                    .checked_add(Uint128::new(1_000_000_000u128))
                    .unwrap(),
            );
        })
        .query_rewards(
            incentive_addr.clone().into_inner(),
            carol.clone(),
            |result| {
                // There's nothing left to claim
                let err = result.unwrap_err();
                match err {
                    StdError::GenericErr { msg } => {
                        assert_eq!(
                            msg.rsplit_once(": ").unwrap().1,
                            (ContractError::NothingToClaim {}).to_string()
                        );
                    }
                    _ => panic!("Unexpected error type"),
                }
            },
        );
}

/// this test tries to recreate a scenario with multiple parties involved in flows.
#[test]
fn open_expand_close_flows_positions_and_claim_native_token_incentive() {
    let mut suite = TestingSuite::default_with_balances(vec![
        coin(100_000_000_000u128, "uwhale".to_string()),
        coin(100_000_000_000u128, "usdc".to_string()),
        coin(100_000_000_000u128, "ampWHALE".to_string()),
    ]);
    let alice = suite.creator();
    let bob = suite.senders[1].clone();
    let carol = suite.senders[2].clone();

    suite.instantiate_default_native_fee();

    let incentive_asset = AssetInfo::Token {
        contract_addr: suite.cw20_tokens.first().unwrap().to_string(),
    };

    let incentive_asset_addr = suite.cw20_tokens.first().unwrap().clone();
    let mut incentive_addr = RefCell::new(Addr::unchecked(""));

    let flow_asset_1 = AssetInfo::NativeToken {
        denom: "ampWHALE".to_string(),
    };

    let flow_asset_2 = AssetInfo::NativeToken {
        denom: "usdc".to_string(),
    };

    suite
        .create_incentive(alice.clone(), incentive_asset.clone(), |result| {
            result.unwrap();
        })
        .query_incentive(incentive_asset.clone(), |result| {
            let incentive = result.unwrap();
            assert!(incentive.is_some());
            *incentive_addr.borrow_mut() = incentive.unwrap();
        });

    // alice creates two flows
    let time = Timestamp::from_seconds(1684766796u64);
    suite.set_time(time);

    let mut current_epoch = RefCell::new(0u64);
    suite
        .create_epochs_on_fee_distributor(10, vec![incentive_addr.clone().into_inner()])
        .query_current_epoch(|result| {
            *current_epoch.borrow_mut() = result.unwrap().epoch.id.u64();
        });

    let mut alice_ampWHALE_funds = RefCell::new(Uint128::zero());
    let mut alice_usdc_funds = RefCell::new(Uint128::zero());
    let mut bob_ampWHALE_funds = RefCell::new(Uint128::zero());
    let mut bob_usdc_funds = RefCell::new(Uint128::zero());
    let mut carol_ampWHALE_funds = RefCell::new(Uint128::zero());
    let mut carol_usdc_funds = RefCell::new(Uint128::zero());

    suite
        .open_incentive_flow(
            alice.clone(),
            incentive_addr.clone().into_inner(),
            None,
            time.plus_seconds(864000u64).seconds(),  //10 days
            None,                                    //epoch 11
            current_epoch.clone().into_inner() + 10, // epoch 21
            Curve::Linear,
            Asset {
                info: flow_asset_1.clone(),
                amount: Uint128::new(1_000_000_000u128),
            },
            &vec![
                coin(1_000_000_000u128, "ampWHALE"),
                coin(1_000u128, "uwhale"),
            ],
            |result| {
                result.unwrap();
            },
        )
        .open_incentive_flow(
            alice.clone(),
            incentive_addr.clone().into_inner(),
            Some(time.plus_seconds(864000u64).seconds()), // start in 10 days, i.e. when the first flow finishes
            time.plus_seconds(2592000u64).seconds(),      // ends in 20 days from the start
            Some(current_epoch.clone().into_inner() + 10), // epoch 21
            current_epoch.clone().into_inner() + 30, //epoch 41 , ends in 30 epochs from the start, i.e. has a duration of 20 epochs
            Curve::Linear,
            Asset {
                info: flow_asset_2.clone(),
                amount: Uint128::new(10_000_000_000u128),
            },
            &vec![coin(10_000_000_000u128, "usdc"), coin(1_000u128, "uwhale")],
            |result| {
                result.unwrap();
            },
        )
        .query_flows(incentive_addr.clone().into_inner(), |result| {
            let flows = result.unwrap();
            println!("flows created:: {:?}", flows);
            assert_eq!(flows.len(), 2);
            assert_eq!(
                flows[0].clone().flow_asset.amount,
                Uint128::new(1_000_000_000u128)
            );
            assert_eq!(
                flows[1].clone().flow_asset.amount,
                Uint128::new(10_000_000_000u128)
            );
        });

    // alice bonds 1k, unbonding 1 day
    // bob bonds 2k, unbonding in 1 day
    // carol bonds 3k, unbonding in 1 day

    let alice_position_1 = incentive::OpenPosition {
        amount: Uint128::new(1_000u128),
        unbonding_duration: 86400u64,
    };
    let bob_position = incentive::OpenPosition {
        amount: Uint128::new(2_000u128),
        unbonding_duration: 86400u64,
    };
    let carol_position = incentive::OpenPosition {
        amount: Uint128::new(3_000u128),
        unbonding_duration: 86400u64,
    };

    // current epoch is 11

    suite
        .increase_allowance(
            alice.clone(),
            incentive_asset_addr.clone(),
            Uint128::new(2_000u128),
            incentive_addr.clone().into_inner(),
        )
        .increase_allowance(
            bob.clone(),
            incentive_asset_addr.clone(),
            Uint128::new(2_000u128),
            incentive_addr.clone().into_inner(),
        )
        .increase_allowance(
            carol.clone(),
            incentive_asset_addr.clone(),
            Uint128::new(3_000u128),
            incentive_addr.clone().into_inner(),
        )
        .open_incentive_position(
            alice.clone(),
            incentive_addr.clone().into_inner(),
            alice_position_1.amount,
            alice_position_1.unbonding_duration,
            None,
            vec![coin(1_000u128, "uwhale".to_string())],
            |result| {
                result.unwrap();
            },
        )
        .open_incentive_position(
            bob.clone(),
            incentive_addr.clone().into_inner(),
            bob_position.amount,
            bob_position.unbonding_duration,
            None,
            vec![coin(1_000u128, "uwhale".to_string())],
            |result| {
                result.unwrap();
            },
        )
        .open_incentive_position(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            carol_position.amount,
            carol_position.unbonding_duration,
            None,
            vec![coin(1_000u128, "uwhale".to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(
            incentive_addr.clone().into_inner(),
            alice.clone(),
            |result| {
                assert_eq!(
                    result.unwrap().positions.first().unwrap(),
                    &incentive::QueryPosition::OpenPosition {
                        amount: Uint128::new(1_000u128),
                        unbonding_duration: alice_position_1.unbonding_duration,
                        weight: Uint128::new(1_000u128),
                    }
                );
            },
        )
        .query_positions(incentive_addr.clone().into_inner(), bob.clone(), |result| {
            assert_eq!(
                result.unwrap().positions.first().unwrap(),
                &incentive::QueryPosition::OpenPosition {
                    amount: Uint128::new(2_000u128),
                    unbonding_duration: bob_position.unbonding_duration,
                    weight: Uint128::new(2_000u128),
                }
            );
        })
        .query_positions(
            incentive_addr.clone().into_inner(),
            carol.clone(),
            |result| {
                assert_eq!(
                    result.unwrap().positions.first().unwrap(),
                    &incentive::QueryPosition::OpenPosition {
                        amount: Uint128::new(3_000u128),
                        unbonding_duration: carol_position.unbonding_duration,
                        weight: Uint128::new(3_000u128),
                    }
                );
            },
        );

    // everybody locked tokens at the 11th epoch, so their first rewards will start at the 12th epoch!

    // move time 5 days, it means the first flow is 5 days in, and the second one will start in 5 days
    let time = suite.get_time();
    suite.set_time(time.plus_seconds(432000u64));
    // move 4 epochs
    suite.create_epochs_on_fee_distributor(5, vec![incentive_addr.clone().into_inner()]);

    // current epoch is 15
    suite.query_current_epoch(|result| {
        println!("-------current epoch: {:?}", result.unwrap().epoch.id.u64());
    });
    // alice has 16.66% of the weight
    // bob has 33.33% of the weight
    // carol has 50% of the weight

    // by then, 50% of the first flow rewards should be available. i.e. 500_000_000u128 ampWHALE

    println!("--------HEEEERE-------");
    // lets query rewards and claim with alice and bob, carol will claim at the end all at once
    suite
        .query_rewards(
            incentive_addr.clone().into_inner(),
            alice.clone(),
            |result| {
                println!("query alice rewards ++++++");
                assert_eq!(
                    result.unwrap().rewards,
                    vec![Asset {
                        info: flow_asset_1.clone(),
                        amount: Uint128::new(83_333_330u128),
                        // amount: Uint128::new(83_333_333u128),
                    },]
                );
            },
        )
        .query_rewards(incentive_addr.clone().into_inner(), bob.clone(), |result| {
            println!("query bob rewards ++++++");
            assert_eq!(
                result.unwrap().rewards,
                vec![Asset {
                    info: flow_asset_1.clone(),
                    amount: Uint128::new(166_666_665u128),
                    // amount: Uint128::new(166_666_666u128),
                },]
            );
        })
        .query_rewards(
            incentive_addr.clone().into_inner(),
            carol.clone(),
            |result| {
                println!("query carol rewards ++++++");
                assert_eq!(
                    result.unwrap().rewards,
                    vec![Asset {
                        info: flow_asset_1.clone(),
                        amount: Uint128::new(250_000_000u128),
                    },]
                );
            },
        )
        .query_funds(alice.clone(), flow_asset_1.clone(), |result| {
            println!("query alice funds {}", result);
            *alice_ampWHALE_funds.borrow_mut() = result;
        })
        .query_funds(bob.clone(), flow_asset_1.clone(), |result| {
            println!("query bob funds {}", result);

            *bob_ampWHALE_funds.borrow_mut() = result;
        })
        .query_funds(carol.clone(), flow_asset_1.clone(), |result| {
            println!("query carol funds {}", result);

            *carol_ampWHALE_funds.borrow_mut() = result;
        })
        .claim(
            incentive_addr.clone().into_inner(),
            alice.clone(),
            |result| {
                result.unwrap();
            },
        )
        .query_funds(alice.clone(), flow_asset_1.clone(), |result| {
            println!("query alice funds again {}", result);

            assert_eq!(
                result,
                alice_ampWHALE_funds
                    .clone()
                    .into_inner()
                    .checked_add(Uint128::new(83_333_330u128))
                    .unwrap(),
            );
        })
        .claim(incentive_addr.clone().into_inner(), bob.clone(), |result| {
            result.unwrap();
        })
        .query_funds(bob.clone(), flow_asset_1.clone(), |result| {
            println!("query bob funds again {}", result);

            assert_eq!(
                result,
                bob_ampWHALE_funds
                    .clone()
                    .into_inner()
                    .checked_add(Uint128::new(166_666_665u128))
                    .unwrap(),
            );
        });

    println!("all good");
    suite.query_flow(incentive_addr.clone().into_inner(), 1u64, |result| {
        let flow_response = result.unwrap();
        assert_eq!(
            flow_response.unwrap().flow.unwrap().claimed_amount,
            // Uint128::new(250_000_000u128)
            Uint128::new(249_999_995u128)
        );
    });

    // move 10 epochs
    let time = suite.get_time();
    suite.set_time(time.plus_seconds(864000u64));
    suite.create_epochs_on_fee_distributor(10, vec![incentive_addr.clone().into_inner()]);

    // current epoch is 26
    suite.query_current_epoch(|result| {
        println!("-------current epoch: {:?}", result.unwrap().epoch.id.u64());
    });

    println!("CAROL IS GONNA CLAIM NOW");
    // now the flow 1 should have finished (in epoch 21) so let's try to claim with all users
    suite
        .query_funds(alice.clone(), flow_asset_2.clone(), |result| {
            *alice_usdc_funds.borrow_mut() = result;
        })
        .query_funds(bob.clone(), flow_asset_2.clone(), |result| {
            *bob_usdc_funds.borrow_mut() = result;
        })
        .query_funds(carol.clone(), flow_asset_2.clone(), |result| {
            *carol_usdc_funds.borrow_mut() = result;
        })
        .query_funds(alice.clone(), flow_asset_1.clone(), |result| {
            *alice_ampWHALE_funds.borrow_mut() = result;
        })
        .query_funds(bob.clone(), flow_asset_1.clone(), |result| {
            *bob_ampWHALE_funds.borrow_mut() = result;
        })
        .query_funds(carol.clone(), flow_asset_1.clone(), |result| {
            *carol_ampWHALE_funds.borrow_mut() = result;
        })
        .query_rewards(
            incentive_addr.clone().into_inner(),
            alice.clone(),
            |result| {
                println!("query alice rewards ++++++");
                assert_eq!(
                    result.unwrap().rewards,
                    vec![
                        Asset {
                            info: flow_asset_1.clone(),
                            amount: Uint128::new(66_666_664u128),
                        },
                        Asset {
                            info: flow_asset_2.clone(),
                            amount: Uint128::new(499_999_998u128),
                        },
                    ]
                );
            },
        )
        .query_rewards(incentive_addr.clone().into_inner(), bob.clone(), |result| {
            println!("query bob rewards ++++++");
            assert_eq!(
                result.unwrap().rewards,
                vec![
                    Asset {
                        info: flow_asset_1.clone(),
                        amount: Uint128::new(133_333_332u128),
                    },
                    Asset {
                        info: flow_asset_2.clone(),
                        amount: Uint128::new(999_999_996u128),
                    },
                ]
            );
        })
        .query_rewards(
            incentive_addr.clone().into_inner(),
            carol.clone(),
            |result| {
                println!("query carol rewards ++++++");
                assert_eq!(
                    result.unwrap().rewards,
                    vec![
                        Asset {
                            info: flow_asset_1.clone(),
                            amount: Uint128::new(450_000_000u128),
                        },
                        Asset {
                            info: flow_asset_2.clone(),
                            amount: Uint128::new(1_500_000_000u128),
                        },
                    ]
                );
            },
        );

    // now that we are in epoch 26, claim everything for everyone.

    suite
        .claim(
            incentive_addr.clone().into_inner(),
            alice.clone(),
            |result| {
                result.unwrap();
            },
        )
        .query_funds(alice.clone(), flow_asset_1.clone(), |result| {
            println!("query alice funds again {}", result);

            assert_eq!(
                result,
                alice_ampWHALE_funds
                    .clone()
                    .into_inner()
                    .checked_add(Uint128::new(66_666_664u128))
                    .unwrap(),
            );
            *alice_ampWHALE_funds.borrow_mut() = result;
        })
        .query_funds(alice.clone(), flow_asset_2.clone(), |result| {
            println!("query alice funds again sss {}", result);

            assert_eq!(
                result,
                alice_usdc_funds
                    .clone()
                    .into_inner()
                    .checked_add(Uint128::new(499_999_998u128))
                    .unwrap(),
            );
            *alice_usdc_funds.borrow_mut() = result;
        })
        .close_incentive_position(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            carol_position.unbonding_duration,
            |result| {
                // trying to close the position with pending rewards should fail

                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::PendingRewards { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::PendingRewards"),
                }
            },
        )
        .claim(
            incentive_addr.clone().into_inner(),
            carol.clone(),
            |result| {
                result.unwrap();
            },
        )
        .query_funds(carol.clone(), flow_asset_1.clone(), |result| {
            println!("query carol funds again {}", result);

            assert_eq!(
                result,
                carol_ampWHALE_funds
                    .clone()
                    .into_inner()
                    .checked_add(Uint128::new(450_000_000u128))
                    .unwrap(),
            );
            *carol_ampWHALE_funds.borrow_mut() = result;
        })
        .query_funds(carol.clone(), flow_asset_2.clone(), |result| {
            println!("query carol funds again sss {}", result);

            assert_eq!(
                result,
                carol_usdc_funds
                    .clone()
                    .into_inner()
                    .checked_add(Uint128::new(1_500_000_000u128))
                    .unwrap(),
            );
            *carol_usdc_funds.borrow_mut() = result;
        })
        .claim(incentive_addr.clone().into_inner(), bob.clone(), |result| {
            result.unwrap();
        })
        .query_funds(bob.clone(), flow_asset_1.clone(), |result| {
            println!("query bob funds again {}", result);

            assert_eq!(
                result,
                bob_ampWHALE_funds
                    .clone()
                    .into_inner()
                    .checked_add(Uint128::new(133_333_332u128))
                    .unwrap(),
            );

            *bob_ampWHALE_funds.borrow_mut() = result;
        })
        .query_funds(bob.clone(), flow_asset_2.clone(), |result| {
            println!("query bob funds again sss {}", result);

            assert_eq!(
                result,
                bob_usdc_funds
                    .clone()
                    .into_inner()
                    .checked_add(Uint128::new(999_999_996u128))
                    .unwrap(),
            );

            *bob_usdc_funds.borrow_mut() = result;
        });

    // now let's expand some positions
    // at this point:
    // alice has 16.66% of the weight
    // bob has 33.33% of the weight
    // carol has 50% of the weight (carol's weight is zero for next epoch (27), already closed it before)

    // let's expand alice's position
    suite
        .expand_incentive_position(
            alice.clone(),
            incentive_addr.clone().into_inner(),
            alice_position_1.amount.clone(),
            alice_position_1.unbonding_duration.clone() + 4,
            None,
            vec![],
            |result| {
                // tried to expand a position that doesn't exist, should return an error
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::NonExistentPosition { .. } => {}
                    _ => {
                        panic!("Wrong error type, should return ContractError::NonExistentPosition")
                    }
                }
            },
        )
        .expand_incentive_position(
            alice.clone(),
            incentive_addr.clone().into_inner(),
            alice_position_1.amount.clone(),
            alice_position_1.unbonding_duration.clone(),
            None,
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .close_incentive_position(
            carol.clone(),
            incentive_addr.clone().into_inner(),
            carol_position.unbonding_duration,
            |result| {
                // closing the position for carol here in epoch 26, which means she will get the rewards for epoch 26 and her new weight of zero
                // will be applied to the next epoch (27)
                result.unwrap();
            },
        );

    // for the next epoch, both alice and bob should be 50% of the weight
    let time = suite.get_time();
    suite.set_time(time.plus_seconds(86400u64));
    suite
        .create_epochs_on_fee_distributor(1, vec![incentive_addr.clone().into_inner()])
        .query_current_epoch(|result| {
            *current_epoch.borrow_mut() = result.unwrap().epoch.id.u64();
        });

    println!("current epoch is now -> {}", current_epoch.into_inner());

    // let's close flow 1
    suite
        .query_flow(incentive_addr.clone().into_inner(), 1u64, |result| {
            let flow_response = result.unwrap();
            let total_rewards = flow_response
                .clone()
                .unwrap()
                .flow
                .unwrap()
                .flow_asset
                .amount;
            let claimed = flow_response.clone().unwrap().flow.unwrap().claimed_amount;
            let expected_claimed = total_rewards - Uint128::new(100_000_000u128);
            assert!(total_rewards > claimed);
            assert!(expected_claimed >= claimed);

            assert!((expected_claimed.u128() as i128 - claimed.u128() as i128).abs() < 10i128);
        })
        .close_incentive_flow(
            bob.clone(),
            incentive_addr.clone().into_inner(),
            1u64,
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::UnauthorizedFlowClose { .. } => {}
                    _ => panic!(
                        "Wrong error type, should return ContractError::UnauthorizedFlowClose"
                    ),
                }
            },
        )
        .close_incentive_flow(
            bob.clone(),
            incentive_addr.clone().into_inner(),
            5u64,
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::NonExistentFlow { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::NonExistentFlow"),
                }
            },
        )
        .close_incentive_flow(
            alice.clone(),
            incentive_addr.clone().into_inner(),
            1u64,
            |result| {
                result.unwrap();
            },
        )
        .query_funds(alice.clone(), flow_asset_1.clone(), |result| {
            println!("the funds that were remaining in the flow went back to alice");
            assert_eq!(
                result,
                alice_ampWHALE_funds
                    .clone()
                    .into_inner()
                    .checked_add(Uint128::new(100_000_009u128)) // the 9 is about the math imprecision
                    .unwrap(),
            );

            *alice_ampWHALE_funds.borrow_mut() = result;
        });

    // current epoch is 27 now

    println!("last round ------");
    suite
        .query_rewards(
            incentive_addr.clone().into_inner(),
            carol.clone(),
            |result| {
                // she should have 0 since she closed her position in epoch 26
                assert_eq!(
                    result.unwrap().rewards,
                    vec![Asset {
                        info: flow_asset_2.clone(),
                        amount: Uint128::zero(),
                    },]
                );
            },
        )
        .query_rewards(
            incentive_addr.clone().into_inner(),
            alice.clone(),
            |result| {
                println!("query alice rewards ++++++");
                // alice has now 50% of the rewards
                assert_eq!(
                    result.unwrap().rewards,
                    vec![Asset {
                        info: flow_asset_2.clone(),
                        amount: Uint128::new(250_000_000u128),
                    },]
                );
            },
        )
        .query_rewards(incentive_addr.clone().into_inner(), bob.clone(), |result| {
            println!("query bob rewards ++++++");
            // bob has now 50% of the rewards
            assert_eq!(
                result.unwrap().rewards,
                vec![Asset {
                    info: flow_asset_2.clone(),
                    amount: Uint128::new(250_000_000u128),
                },]
            );
        });
    // move to epoch 37

    let time = suite.get_time();
    suite.set_time(time.plus_seconds(8640000u64));
    suite
        .create_epochs_on_fee_distributor(10, vec![incentive_addr.clone().into_inner()])
        .query_rewards(
            incentive_addr.clone().into_inner(),
            carol.clone(),
            |result| {
                // she should have 0 since she closed her position in epoch 26
                assert_eq!(
                    result.unwrap().rewards,
                    vec![Asset {
                        info: flow_asset_2.clone(),
                        amount: Uint128::zero(),
                    },]
                );
            },
        )
        .query_rewards(
            incentive_addr.clone().into_inner(),
            alice.clone(),
            |result| {
                println!("query alice rewards ++++++");
                // alice has now 50% of the rewards
                assert_eq!(
                    result.unwrap().rewards,
                    vec![Asset {
                        info: flow_asset_2.clone(),
                        amount: Uint128::new(2_750_000_000u128),
                    },]
                );
            },
        )
        .query_rewards(incentive_addr.clone().into_inner(), bob.clone(), |result| {
            println!("query bob rewards ++++++");
            // bob has now 50% of the rewards
            assert_eq!(
                result.unwrap().rewards,
                vec![Asset {
                    info: flow_asset_2.clone(),
                    amount: Uint128::new(2_750_000_000u128),
                },]
            );
        })
        .claim(incentive_addr.clone().into_inner(), bob.clone(), |result| {
            result.unwrap();
        })
        .query_funds(bob.clone(), flow_asset_2.clone(), |result| {
            assert_eq!(
                result,
                bob_usdc_funds
                    .clone()
                    .into_inner()
                    .checked_add(Uint128::new(2_750_000_000u128))
                    .unwrap(),
            );
            *bob_usdc_funds.borrow_mut() = result;
        })
        .claim(
            incentive_addr.clone().into_inner(),
            alice.clone(),
            |result| {
                result.unwrap();
            },
        )
        .query_funds(alice.clone(), flow_asset_2.clone(), |result| {
            assert_eq!(
                result,
                alice_usdc_funds
                    .clone()
                    .into_inner()
                    .checked_add(Uint128::new(2_750_000_000u128))
                    .unwrap(),
            );
            *alice_usdc_funds.borrow_mut() = result;
        });

    // let's open another position for bob, with larger unbonding time
    // his weight will no longer be 50% but around 66%, so alice has around 33%
    // this second position also has longer unbonding duration, which gives him more rewards

    let bob_position_2 = incentive::OpenPosition {
        amount: Uint128::new(2_000u128),
        unbonding_duration: 259200u64, //86400u64 * 3
    };

    println!("xxxxxxxxxxxxxxxxx");
    suite
        .increase_allowance(
            bob.clone(),
            incentive_asset_addr.clone(),
            Uint128::new(2_000u128),
            incentive_addr.clone().into_inner(),
        )
        .open_incentive_position(
            bob.clone(),
            incentive_addr.clone().into_inner(),
            bob_position_2.amount.clone(),
            bob_position_2.unbonding_duration.clone(),
            None,
            vec![],
            |result| {
                result.unwrap();
            },
        );
    // move to epoch 47, beyond the end of the flow 2
    let time = suite.get_time();
    suite.set_time(time.plus_seconds(864000u64));
    suite
        .create_epochs_on_fee_distributor(10, vec![incentive_addr.clone().into_inner()])
        .query_rewards(incentive_addr.clone().into_inner(), bob.clone(), |result| {
            assert_eq!(
                result.unwrap().rewards,
                vec![Asset {
                    info: flow_asset_2.clone(),
                    amount: Uint128::new(1_000_998_003u128),
                },]
            );
        })
        .query_rewards(
            incentive_addr.clone().into_inner(),
            alice.clone(),
            |result| {
                assert_eq!(
                    result.unwrap().rewards,
                    vec![Asset {
                        info: flow_asset_2.clone(),
                        amount: Uint128::new(499_001_994u128),
                    },]
                );
            },
        )
        // 499_001_994u128 + 1_000_998_003u128 = 1_499_999_997, sweet!
        .claim(incentive_addr.clone().into_inner(), bob.clone(), |result| {
            result.unwrap();
        })
        .query_funds(bob.clone(), flow_asset_2.clone(), |result| {
            assert_eq!(
                result,
                bob_usdc_funds
                    .clone()
                    .into_inner()
                    .checked_add(Uint128::new(1_000_998_003u128))
                    .unwrap(),
            );
            *bob_usdc_funds.borrow_mut() = result;
        })
        .claim(
            incentive_addr.clone().into_inner(),
            alice.clone(),
            |result| {
                result.unwrap();
            },
        )
        .query_funds(alice.clone(), flow_asset_2.clone(), |result| {
            assert_eq!(
                result,
                alice_usdc_funds
                    .clone()
                    .into_inner()
                    .checked_add(Uint128::new(499_001_994u128))
                    .unwrap(),
            );
            *alice_usdc_funds.borrow_mut() = result;
        })
        .query_flow(incentive_addr.clone().into_inner(), 2u64, |result| {
            let flow_response = result.unwrap();
            let total_rewards = flow_response
                .clone()
                .unwrap()
                .flow
                .unwrap()
                .flow_asset
                .amount;
            let claimed = flow_response.clone().unwrap().flow.unwrap().claimed_amount;
            let expected_claimed = total_rewards.clone();

            assert!(total_rewards > claimed);
            assert!(expected_claimed >= claimed);
            assert!((expected_claimed.u128() as i128 - claimed.u128() as i128).abs() < 10i128);
        });

    // carol should be able to withdraw, as many epochs has passed
    let carol_incentive_asset_funds = RefCell::new(Uint128::zero());
    suite
        .query_funds(carol.clone(), incentive_asset.clone(), |result| {
            *carol_incentive_asset_funds.borrow_mut() = result;
            println!(
                "carol_incentive_asset_funds {:?}",
                carol_incentive_asset_funds.clone().into_inner()
            );
        })
        .withdraw(
            incentive_addr.clone().into_inner(),
            carol.clone(),
            |result| {
                result.unwrap();
            },
        )
        .query_funds(carol.clone(), incentive_asset.clone(), |result| {
            assert_eq!(
                result,
                carol_incentive_asset_funds
                    .clone()
                    .into_inner()
                    .checked_add(Uint128::new(carol_position.amount.u128()))
                    .unwrap(),
            );
            *carol_incentive_asset_funds.borrow_mut() = result;
        })
        // try withdrawing again, nothing should happen as she doesn't have more closed positions
        .withdraw(
            incentive_addr.clone().into_inner(),
            carol.clone(),
            |result| {
                result.unwrap();
            },
        )
        .query_funds(carol.clone(), incentive_asset.clone(), |result| {
            assert_eq!(result, carol_incentive_asset_funds.clone().into_inner(),);
        });
}

#[test]
fn take_global_weight_snapshot() {
    let mut suite = TestingSuite::default_with_balances(vec![]);
    let alice = suite.creator();

    suite.instantiate_default_native_fee();

    let incentive_asset = AssetInfo::Token {
        contract_addr: suite.cw20_tokens.first().unwrap().to_string(),
    };

    let mut incentive_addr = RefCell::new(Addr::unchecked(""));

    suite
        .create_incentive(alice.clone(), incentive_asset.clone(), |result| {
            result.unwrap();
        })
        .query_incentive(incentive_asset.clone(), |result| {
            let incentive = result.unwrap();
            assert!(incentive.is_some());
            *incentive_addr.borrow_mut() = incentive.unwrap();
        })
        .take_global_weight_snapshot(incentive_addr.clone().into_inner(), |result| {
            result.unwrap();
        })
        .take_global_weight_snapshot(incentive_addr.clone().into_inner(), |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();

            match err {
                ContractError::GlobalWeightSnapshotAlreadyExists { epoch } => assert_eq!(epoch, 1),
                _ => panic!(
                    "Wrong error type, should return ContractError::GlobalWeightSnapshotAlreadyExists"
                ),
            }
        })
    ;
}
