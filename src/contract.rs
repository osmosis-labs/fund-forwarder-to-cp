use cosmwasm_std::{
    entry_point, CosmosMsg, DepsMut, DistributionMsg, Env, MessageInfo, Response,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{Config, CONFIG};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        denom: msg.denom,
    };
    CONFIG.save(deps.storage, &config)?;
    
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("denom", &config.denom))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ForwardFunds {} => execute_forward_funds(deps, env, info),
    }
}

pub fn execute_forward_funds(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    
    let balance = deps.querier.query_balance(&env.contract.address, &config.denom)?;
    
    if balance.amount.is_zero() {
        return Err(ContractError::NoFunds {});
    }

    let msg = CosmosMsg::Distribution(DistributionMsg::FundCommunityPool {
        amount: vec![balance.clone()],
    });

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("method", "forward_funds")
        .add_attribute("sender", info.sender)
        .add_attribute("contract", &env.contract.address)
        .add_attribute("amount_forwarded", balance.amount)
        .add_attribute("denom", &config.denom))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, message_info};
    use cosmwasm_std::{coins, Coin, Uint128};

    const USDN_DENOM: &str = "ibc/0C39BD03B5C57A1753A9B73164705871A9B549F1A5226CFD7E39BE7BF73CF8CF";

    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            denom: USDN_DENOM.to_string(),
        };
        let info = message_info(&deps.api.addr_make("creator"), &coins(1000, "earth"));
        let env = mock_env();

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(1, res.attributes.len());
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "instantiate");
        assert_eq!(res.attributes[1].key, "denom");
        assert_eq!(res.attributes[1].value, USDN_DENOM);
    }

    #[test]
    fn forward_funds_success() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&deps.api.addr_make("anyone"), &[]);

        // Mock the contract having USDN balance
        deps.querier.bank.update_balance(
            env.contract.address.clone(),
            vec![Coin {
                denom: USDN_DENOM.to_string(),
                amount: Uint128::new(1000000),
            }],
        );

        let msg = ExecuteMsg::ForwardFunds {};
        let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Check the message was created
        assert_eq!(res.messages.len(), 1);
        
        // Verify it's a FundCommunityPool message
        match &res.messages[0].msg {
            CosmosMsg::Distribution(DistributionMsg::FundCommunityPool { amount }) => {
                assert_eq!(amount.len(), 1);
                assert_eq!(amount[0].denom, USDN_DENOM);
                assert_eq!(amount[0].amount, Uint128::new(1000000));
            }
            _ => panic!("Expected FundCommunityPool message"),
        }

        // Check attributes
        assert_eq!(res.attributes.len(), 5);
        assert_eq!(res.attributes[0].value, "forward_funds");
        assert_eq!(res.attributes[3].key, "amount_forwarded");
        assert_eq!(res.attributes[3].value, "1000000");
        assert_eq!(res.attributes[4].key, "denom");
        assert_eq!(res.attributes[4].value, USDN_DENOM);
    }

    #[test]
    fn forward_funds_no_balance() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&deps.api.addr_make("anyone"), &[]);

        // Contract has no USDN balance
        let msg = ExecuteMsg::ForwardFunds {};
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();

        match err {
            ContractError::NoFunds {} => {}
            _ => panic!("Expected NoFunds error"),
        }
    }

    #[test]
    fn forward_funds_zero_balance() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&deps.api.addr_make("anyone"), &[]);

        // Mock the contract having zero USDN balance
        deps.querier.bank.update_balance(
            env.contract.address.clone(),
            vec![Coin {
                denom: USDN_DENOM.to_string(),
                amount: Uint128::zero(),
            }],
        );

        let msg = ExecuteMsg::ForwardFunds {};
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();

        match err {
            ContractError::NoFunds {} => {}
            _ => panic!("Expected NoFunds error"),
        }
    }

    #[test]
    fn forward_funds_ignores_other_tokens() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&deps.api.addr_make("anyone"), &[]);

        // Mock the contract having other tokens but no USDN
        deps.querier.bank.update_balance(
            env.contract.address.clone(),
            vec![
                Coin {
                    denom: "uosmo".to_string(),
                    amount: Uint128::new(5000000),
                },
                Coin {
                    denom: "uatom".to_string(),
                    amount: Uint128::new(2000000),
                },
            ],
        );

        let msg = ExecuteMsg::ForwardFunds {};
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();

        match err {
            ContractError::NoFunds {} => {}
            _ => panic!("Expected NoFunds error"),
        }
    }
}