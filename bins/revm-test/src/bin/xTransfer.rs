use std::{io::Read, str::FromStr};

use alloy_sol_macro::sol;
use alloy_sol_types::{sol_data::Address, SolCall, SolInterface, SolType};
use revm::{
    db::{CacheDB, EmptyDB},
    primitives::{
        address, keccak256, ruint::Uint, AccountInfo, Bytecode, Bytes, ChainAddress,
        ExecutionResult, OnChain, Output, TransactTo, B256, KECCAK_EMPTY, U256,
    },
    Database, Evm,
};

sol! {
    // Compiled with constructor parameter = 99999
    #[sol(bytecode="0x6080604052348015600e575f80fd5b5060405161172e38038061172e8339818101604052810190602e919060a6565b805f803373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20819055505060cc565b5f80fd5b5f819050919050565b6088816078565b81146091575f80fd5b50565b5f8151905060a0816081565b92915050565b5f6020828403121560b85760b76074565b5b5f60c3848285016094565b91505092915050565b611655806100d95f395ff3fe608060405234801561000f575f80fd5b50600436106100a7575f3560e01c80634e6ec2471161006f5780634e6ec2471461019b57806370a08231146101cb578063a9059cbb146101fb578063b3f322af1461022b578063d48024a71461025b578063dd62ed3e1461028b576100a7565b8063095ea7b3146100ab578063104e81ff146100db57806323b872dd1461010b578063297926631461013b57806330e0789e1461016b575b5f80fd5b6100c560048036038101906100c09190610f73565b6102bb565b6040516100d29190610fc0565b60405180910390f35b6100f560048036038101906100f09190610fd9565b6103a7565b6040516101029190610fc0565b60405180910390f35b61012560048036038101906101209190610fd9565b610502565b6040516101329190610fc0565b60405180910390f35b61015560048036038101906101509190611029565b610846565b6040516101629190610fc0565b60405180910390f35b61018560048036038101906101809190610fd9565b6108d6565b6040516101929190610fc0565b60405180910390f35b6101b560048036038101906101b09190610f73565b6109f3565b6040516101c29190610fc0565b60405180910390f35b6101e560048036038101906101e09190611079565b610abd565b6040516101f29190610fc0565b60405180910390f35b61021560048036038101906102109190610f73565b610ad1565b6040516102229190610fc0565b60405180910390f35b61024560048036038101906102409190611029565b610c63565b6040516102529190610fc0565b60405180910390f35b61027560048036038101906102709190611029565b610cf5565b6040516102829190610fc0565b60405180910390f35b6102a560048036038101906102a091906110a4565b610dd5565b6040516102b29190610fc0565b60405180910390f35b5f8160015f3373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20819055508273ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925846040516103969190610fc0565b60405180910390a381905092915050565b5f3073ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff1614610416576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161040d90611162565b60405180910390fd5b8160015f8673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20819055508273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925846040516104f09190610fc0565b60405180910390a38190509392505050565b5f815f808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20541015610582576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610579906111ca565b60405180910390fd5b3373ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff1614610671578160015f8673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f3373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20541015610670576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161066790611232565b60405180910390fd5b5b815f808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8282546106bc919061127d565b92505081905550815f808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f82825461070e91906112b0565b925050819055503373ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff16146107d7578160015f8673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f3373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8282546107cf919061127d565b925050819055505b8273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef846040516108349190610fc0565b60405180910390a38190509392505050565b5f61085084610df5565b3073ffffffffffffffffffffffffffffffffffffffff1663104e81ff3385856040518463ffffffff1660e01b815260040161088d939291906112f2565b6020604051808303815f875af11580156108a9573d5f803e3d5ffd5b505050506040513d601f19601f820116820180604052508101906108cd919061133b565b90509392505050565b5f3073ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff1614610945576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161093c906113b0565b60405180910390fd5b815f808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f828254610990919061127d565b92505081905550815f808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8282546109e291906112b0565b925050819055508190509392505050565b5f3073ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff1614610a62576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610a59906113b0565b60405180910390fd5b815f808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f828254610aad91906112b0565b9250508190555081905092915050565b5f602052805f5260405f205f915090505481565b5f815f803373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20541015610b51576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610b48906111ca565b60405180910390fd5b815f803373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f828254610b9c919061127d565b92505081905550815f808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f828254610bee91906112b0565b925050819055508273ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef84604051610c529190610fc0565b60405180910390a381905092915050565b5f610c6f846001610e02565b3073ffffffffffffffffffffffffffffffffffffffff166330e0789e3385856040518463ffffffff1660e01b8152600401610cac939291906112f2565b6020604051808303815f875af1158015610cc8573d5f803e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610cec919061133b565b90509392505050565b5f815f803373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f828254610d41919061127d565b92505081905550610d5184610df5565b3073ffffffffffffffffffffffffffffffffffffffff16634e6ec24784846040518363ffffffff1660e01b8152600401610d8c9291906113ce565b6020604051808303815f875af1158015610da8573d5f803e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610dcc919061133b565b90509392505050565b6001602052815f5260405f20602052805f5260405f205f91509150505481565b610dff815f610e02565b50565b610e0e82823230610e12565b5050565b610e30848484845f801b60405180602001604052805f815250610e36565b50505050565b5f6001878787878787604051602001610e55979695949392919061158d565b60405160208183030381529060405290505f6104d273ffffffffffffffffffffffffffffffffffffffff1682604051610e8e9190611609565b5f60405180830381855afa9150503d805f8114610ec6576040519150601f19603f3d011682016040523d82523d5f602084013e610ecb565b606091505b5050905080610ed8575f80fd5b5050505050505050565b5f80fd5b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f610f0f82610ee6565b9050919050565b610f1f81610f05565b8114610f29575f80fd5b50565b5f81359050610f3a81610f16565b92915050565b5f819050919050565b610f5281610f40565b8114610f5c575f80fd5b50565b5f81359050610f6d81610f49565b92915050565b5f8060408385031215610f8957610f88610ee2565b5b5f610f9685828601610f2c565b9250506020610fa785828601610f5f565b9150509250929050565b610fba81610f40565b82525050565b5f602082019050610fd35f830184610fb1565b92915050565b5f805f60608486031215610ff057610fef610ee2565b5b5f610ffd86828701610f2c565b935050602061100e86828701610f2c565b925050604061101f86828701610f5f565b9150509250925092565b5f805f606084860312156110405761103f610ee2565b5b5f61104d86828701610f5f565b935050602061105e86828701610f2c565b925050604061106f86828701610f5f565b9150509250925092565b5f6020828403121561108e5761108d610ee2565b5b5f61109b84828501610f2c565b91505092915050565b5f80604083850312156110ba576110b9610ee2565b5b5f6110c785828601610f2c565b92505060206110d885828601610f2c565b9150509250929050565b5f82825260208201905092915050565b7f4f6e6c7920636f6e747261637420697473656c662063616e2063616c6c2074685f8201527f69732066756e6374696f6e000000000000000000000000000000000000000000602082015250565b5f61114c602b836110e2565b9150611157826110f2565b604082019050919050565b5f6020820190508181035f83015261117981611140565b9050919050565b7f496e73756666696369656e742062616c616e63650000000000000000000000005f82015250565b5f6111b46014836110e2565b91506111bf82611180565b602082019050919050565b5f6020820190508181035f8301526111e1816111a8565b9050919050565b7f416c6c6f77616e636520657863656564656400000000000000000000000000005f82015250565b5f61121c6012836110e2565b9150611227826111e8565b602082019050919050565b5f6020820190508181035f83015261124981611210565b9050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b5f61128782610f40565b915061129283610f40565b92508282039050818111156112aa576112a9611250565b5b92915050565b5f6112ba82610f40565b91506112c583610f40565b92508282019050808211156112dd576112dc611250565b5b92915050565b6112ec81610f05565b82525050565b5f6060820190506113055f8301866112e3565b61131260208301856112e3565b61131f6040830184610fb1565b949350505050565b5f8151905061133581610f49565b92915050565b5f602082840312156113505761134f610ee2565b5b5f61135d84828501611327565b91505092915050565b7f4f6e6c79207468697320636f6e74726163742063616e206d696e7400000000005f82015250565b5f61139a601b836110e2565b91506113a582611366565b602082019050919050565b5f6020820190508181035f8301526113c78161138e565b9050919050565b5f6040820190506113e15f8301856112e3565b6113ee6020830184610fb1565b9392505050565b5f61ffff82169050919050565b5f8160f01b9050919050565b5f61141882611402565b9050919050565b61143061142b826113f5565b61140e565b82525050565b5f67ffffffffffffffff82169050919050565b5f8160c01b9050919050565b5f61145f82611449565b9050919050565b61147761147282611436565b611455565b82525050565b5f8115159050919050565b5f8160f81b9050919050565b5f61149e82611488565b9050919050565b5f6114af82611494565b9050919050565b6114c76114c28261147d565b6114a5565b82525050565b5f8160601b9050919050565b5f6114e3826114cd565b9050919050565b5f6114f4826114d9565b9050919050565b61150c61150782610f05565b6114ea565b82525050565b5f819050919050565b5f819050919050565b61153561153082611512565b61151b565b82525050565b5f81519050919050565b5f81905092915050565b8281835e5f83830152505050565b5f6115678261153b565b6115718185611545565b935061158181856020860161154f565b80840191505092915050565b5f611598828a61141f565b6002820191506115a88289611466565b6008820191506115b882886114b6565b6001820191506115c882876114fb565b6014820191506115d882866114fb565b6014820191506115e88285611524565b6020820191506115f8828461155d565b915081905098975050505050505050565b5f611614828461155d565b91508190509291505056fea26469706673582212202a5a89b8d8a4c6b167336eac0fc8db31b6ed25b822365c7988114145eaacc42064736f6c634300081a0033000000000000000000000000000000000000000000000000000000000001869f")]
    contract ERC20 {

        mapping(address => uint256) public balanceOf;
        mapping(address => mapping(address => uint256)) public allowance;

        event Transfer(address indexed from, address indexed to, uint256 value);
        event Approval(address indexed owner, address indexed spender, uint256 value);

        constructor(uint256 totalSupply) {
            balanceOf[msg.sender] = totalSupply;
        }

        // ============ Transfer ============

        function transfer(address to, uint256 value) public returns (uint256) {
            require(balanceOf[msg.sender] >= value, "Insufficient balance");
            balanceOf[msg.sender] -= value;
            balanceOf[to] += value;
            emit Transfer(msg.sender, to, value);
            return value;
        }

        function _transfer(address from, address to, uint256 value) public returns (uint256) {
            require(msg.sender == address(this), "Only this contract can mint");
            balanceOf[from] -= value;
            balanceOf[to] += value;
            return value;
        }

        function _mint(address to, uint256 value) public returns (uint256) {
            require(msg.sender == address(this), "Only this contract can mint");
            balanceOf[to] += value;
            return value;
        }

        function xTransfer(uint256 chain, address to, uint256 value) public returns (uint256) {
            balanceOf[msg.sender] -= value;
            EVM.xCallOptions(chain);
            return this._mint(to, value);
        }

        function sandboxedTransfer(uint256 chain, address to, uint256 value) public returns (uint256) {
            EVM.xCallOptions(chain, true);
            return this._transfer(msg.sender, to, value);
        }

        // ============ Approve ============

        function approve(address spender, uint256 value) public returns (uint256) {
            allowance[msg.sender][spender] = value;
            emit Approval(msg.sender, spender, value);
            return value;
        }

        function _approve(address owner, address spender, uint256 value) public returns (uint256) {
            require(msg.sender == address(this), "Only contract itself can call this function");
            allowance[owner][spender] = value;
            emit Approval(owner, spender, value);
            return value;
        }

        function xApprove(uint256 chain, address spender, uint256 value) public returns (uint256) {
            EVM.xCallOptions(chain);
            return this._approve(msg.sender, spender, value);
        }

        function transferFrom(address from, address to, uint256 value) public returns (uint256) {
            require(balanceOf[from] >= value, "Insufficient balance");
            if (from != msg.sender) {
                require(allowance[from][msg.sender] >= value, "Allowance exceeded");
            }
            balanceOf[from] -= value;
            balanceOf[to] += value;
            if (from != msg.sender) {
                allowance[from][msg.sender] -= value;
            }
            emit Transfer(from, to, value);
            return value;
        }
    }
}

const A: u64 = 1;
const B: u64 = 160010;

/// From A, Alice xTransfer her money to Bob on B.
/// From B, Bob xApprove Alice to spend his on A.
/// From B, Bob checks if he transfer his A money to Alice on A in snadboxed mode.
/// From A, Alice transfer Bob's money to herself.
fn main() {
    let alice = address!("2222000000000000000000000000000000000000");
    let bob = address!("3333000000000000000000000000000000000000");
    let operator = address!("4444000000000000000000000000000000000000");

    let deployment = address!("37ab31eed8a6ae736a28d1371d41ff9dc2c21d37");

    let mut db = CacheDB::new(EmptyDB::default());
    db.insert_account_info(
        alice.on_chain(B),
        AccountInfo::new(U256::MAX, 0, KECCAK_EMPTY, Bytecode::default()),
    );
    db.insert_account_info(
        bob.on_chain(A),
        AccountInfo::new(U256::MAX, 0, KECCAK_EMPTY, Bytecode::default()),
    );
    db.insert_account_info(
        operator.on_chain(B),
        AccountInfo::new(U256::MAX, 0, KECCAK_EMPTY, Bytecode::default()),
    );
    db.insert_account_info(
        operator.on_chain(A),
        AccountInfo::new(U256::MAX, 0, KECCAK_EMPTY, Bytecode::default()),
    );
    deploy_contract(
        &mut db,
        operator.on_chain(A),
        deployment.on_chain(A),
        ERC20::BYTECODE.clone(),
    );
    deploy_contract(
        &mut db,
        operator.on_chain(B),
        deployment.on_chain(B),
        ERC20::BYTECODE.clone(),
    );

    let mut do_transact = |addr: ChainAddress, op: Vec<u8>| -> ExecutionResult {
        println!("\n\n");
        let mut evm = Evm::builder()
            .modify_tx_env(|tx| {
                tx.caller = addr;
                tx.transact_to = TransactTo::Call(ChainAddress(addr.0, deployment));
                tx.data = op.clone().into();
            })
            .with_db(&mut db)
            .build();
        let res = evm.transact().unwrap().result;
        evm.transact_commit().unwrap();
        drop(evm);
        println!("{:?}\n~~~~~~~~~\n", res);
        res
    };

    let results = vec![
        // Give Alice some money
        do_transact(
            operator.on_chain(A),
            ERC20::transferCall {
                to: alice,
                value: Uint::from(1234),
            }
            .abi_encode(),
        ),
        do_transact(
            operator.on_chain(B),
            ERC20::transferCall {
                to: alice,
                value: Uint::from(9876),
            }
            .abi_encode(),
        ),
        do_transact(
            operator.on_chain(A),
            ERC20::transferCall {
                to: bob,
                value: Uint::from(9876),
            }
            .abi_encode(),
        ),
        // Give Bob some money
        do_transact(
            operator.on_chain(B),
            ERC20::transferCall {
                to: bob,
                value: Uint::from(1234),
            }
            .abi_encode(),
        ),
        // From A, Alice xTransfer 666 to Bob on B.
        do_transact(
            alice.on_chain(A),
            ERC20::xTransferCall {
                chain: Uint::from(B),
                to: bob,
                value: Uint::from(666),
            }
            .abi_encode(),
        ),
        // From B, Bob xApprove Alice to spend his 666 on A.
        do_transact(
            bob.on_chain(B),
            ERC20::xApproveCall {
                chain: Uint::from(A),
                spender: alice,
                value: Uint::from(666),
            }
            .abi_encode(),
        ),
        // From B, Bob checks if he transfer 555 to Alice on A in sandboxed mode.
        do_transact(
            bob.on_chain(B),
            ERC20::sandboxedTransferCall {
                chain: Uint::from(A),
                to: alice,
                value: Uint::from(666),
            }
            .abi_encode(),
        ),
        // From A, Alice transfer Bob's 666 to herself.
        do_transact(
            alice.on_chain(A),
            ERC20::transferFromCall {
                from: bob,
                to: alice,
                value: Uint::from(666),
            }
            .abi_encode(),
        ),
    ];

    for res in results {
        assert!(res.is_success());
    }

    println!("Success");
}

fn deploy_contract(
    cache_db: &mut CacheDB<EmptyDB>,
    deployer: ChainAddress,
    addr: ChainAddress,
    code: Bytes,
) {
    let mut evm = Evm::builder()
        .modify_tx_env(|tx| {
            tx.caller = deployer;
            tx.transact_to = TransactTo::Create;
            tx.data = code;
        })
        .with_db(cache_db)
        .build();
    assert!(evm.transact().unwrap().result.is_success());
    evm.transact_commit().unwrap();
    drop(evm);
}
