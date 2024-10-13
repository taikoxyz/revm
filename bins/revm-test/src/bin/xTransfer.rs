use std::{io::Read, str::FromStr};

use alloy_sol_macro::sol;
use alloy_sol_types::{sol_data::Address, SolCall, SolInterface, SolType};
use revm::{
    db::{CacheDB, EmptyDB}, primitives::{
        OnChain,
        address, keccak256, ruint::Uint, AccountInfo, Bytecode, Bytes, ChainAddress, ExecutionResult, Output, TransactTo, B256, KECCAK_EMPTY, U256
    }, Database, Evm
};


sol! {
    #[sol(bytecode="0x608060405234801561000f575f80fd5b5060043610610091575f3560e01c806370a082311161006457806370a0823114610155578063a9059cbb14610185578063b144adfb146101b5578063d9f8bfde146101e5578063dd62ed3e1461021557610091565b8063095ea7b31461009557806323b872dd146100c557806329792663146100f55780633ae7723d14610125575b5f80fd5b6100af60048036038101906100aa9190610b59565b610245565b6040516100bc9190610bb1565b60405180910390f35b6100df60048036038101906100da9190610bca565b610332565b6040516100ec9190610bb1565b60405180910390f35b61010f600480360381019061010a9190610c1a565b610677565b60405161011c9190610bb1565b60405180910390f35b61013f600480360381019061013a9190610c1a565b610694565b60405161014c9190610bb1565b60405180910390f35b61016f600480360381019061016a9190610c6a565b6106b3565b60405161017c9190610ca4565b60405180910390f35b61019f600480360381019061019a9190610b59565b6106c7565b6040516101ac9190610bb1565b60405180910390f35b6101cf60048036038101906101ca9190610c6a565b61085a565b6040516101dc9190610ca4565b60405180910390f35b6101ff60048036038101906101fa9190610cbd565b61089f565b60405161020c9190610ca4565b60405180910390f35b61022f600480360381019061022a9190610d46565b6109bb565b60405161023c9190610ca4565b60405180910390f35b5f8160015f3373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20819055508273ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925846040516103209190610ca4565b60405180910390a36001905092915050565b5f815f808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205410156103b2576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016103a990610dde565b60405180910390fd5b3373ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff16146104a1578160015f8673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f3373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205410156104a0576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161049790610e46565b60405180910390fd5b5b815f808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8282546104ec9190610e91565b92505081905550815f808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f82825461053e9190610ec4565b925050819055503373ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff1614610607578160015f8673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f3373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8282546105ff9190610e91565b925050819055505b8273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef846040516106649190610ca4565b60405180910390a3600190509392505050565b5f610681846109db565b61068b8383610245565b90509392505050565b5f6106a08460016109e8565b6106aa83836106c7565b90509392505050565b5f602052805f5260405f205f915090505481565b5f815f803373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20541015610747576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161073e90610dde565b60405180910390fd5b815f803373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8282546107929190610e91565b92505081905550815f808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8282546107e49190610ec4565b925050819055508273ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef846040516108489190610ca4565b60405180910390a36001905092915050565b5f805f8373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20549050919050565b5f6108a9876109db565b3073ffffffffffffffffffffffffffffffffffffffff166323b872dd8685856040518463ffffffff1660e01b81526004016108e693929190610f06565b6020604051808303815f875af1158015610902573d5f803e3d5ffd5b505050506040513d601f19601f820116820180604052508101906109269190610f65565b50610930866109db565b3073ffffffffffffffffffffffffffffffffffffffff166323b872dd8486856040518463ffffffff1660e01b815260040161096d93929190610f06565b6020604051808303815f875af1158015610989573d5f803e3d5ffd5b505050506040513d601f19601f820116820180604052508101906109ad9190610f65565b508190509695505050505050565b6001602052815f5260405f20602052805f5260405f205f91509150505481565b6109e5815f6109e8565b50565b6109f4828232306109f8565b5050565b610a16848484845f801b60405180602001604052805f815250610a1c565b50505050565b5f6001878787878787604051602001610a3b979695949392919061111d565b60405160208183030381529060405290505f6104d273ffffffffffffffffffffffffffffffffffffffff1682604051610a749190611199565b5f60405180830381855afa9150503d805f8114610aac576040519150601f19603f3d011682016040523d82523d5f602084013e610ab1565b606091505b5050905080610abe575f80fd5b5050505050505050565b5f80fd5b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f610af582610acc565b9050919050565b610b0581610aeb565b8114610b0f575f80fd5b50565b5f81359050610b2081610afc565b92915050565b5f819050919050565b610b3881610b26565b8114610b42575f80fd5b50565b5f81359050610b5381610b2f565b92915050565b5f8060408385031215610b6f57610b6e610ac8565b5b5f610b7c85828601610b12565b9250506020610b8d85828601610b45565b9150509250929050565b5f8115159050919050565b610bab81610b97565b82525050565b5f602082019050610bc45f830184610ba2565b92915050565b5f805f60608486031215610be157610be0610ac8565b5b5f610bee86828701610b12565b9350506020610bff86828701610b12565b9250506040610c1086828701610b45565b9150509250925092565b5f805f60608486031215610c3157610c30610ac8565b5b5f610c3e86828701610b45565b9350506020610c4f86828701610b12565b9250506040610c6086828701610b45565b9150509250925092565b5f60208284031215610c7f57610c7e610ac8565b5b5f610c8c84828501610b12565b91505092915050565b610c9e81610b26565b82525050565b5f602082019050610cb75f830184610c95565b92915050565b5f805f805f8060c08789031215610cd757610cd6610ac8565b5b5f610ce489828a01610b45565b9650506020610cf589828a01610b45565b9550506040610d0689828a01610b12565b9450506060610d1789828a01610b12565b9350506080610d2889828a01610b12565b92505060a0610d3989828a01610b45565b9150509295509295509295565b5f8060408385031215610d5c57610d5b610ac8565b5b5f610d6985828601610b12565b9250506020610d7a85828601610b12565b9150509250929050565b5f82825260208201905092915050565b7f496e73756666696369656e742062616c616e63650000000000000000000000005f82015250565b5f610dc8601483610d84565b9150610dd382610d94565b602082019050919050565b5f6020820190508181035f830152610df581610dbc565b9050919050565b7f416c6c6f77616e636520657863656564656400000000000000000000000000005f82015250565b5f610e30601283610d84565b9150610e3b82610dfc565b602082019050919050565b5f6020820190508181035f830152610e5d81610e24565b9050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b5f610e9b82610b26565b9150610ea683610b26565b9250828203905081811115610ebe57610ebd610e64565b5b92915050565b5f610ece82610b26565b9150610ed983610b26565b9250828201905080821115610ef157610ef0610e64565b5b92915050565b610f0081610aeb565b82525050565b5f606082019050610f195f830186610ef7565b610f266020830185610ef7565b610f336040830184610c95565b949350505050565b610f4481610b97565b8114610f4e575f80fd5b50565b5f81519050610f5f81610f3b565b92915050565b5f60208284031215610f7a57610f79610ac8565b5b5f610f8784828501610f51565b91505092915050565b5f61ffff82169050919050565b5f8160f01b9050919050565b5f610fb382610f9d565b9050919050565b610fcb610fc682610f90565b610fa9565b82525050565b5f67ffffffffffffffff82169050919050565b5f8160c01b9050919050565b5f610ffa82610fe4565b9050919050565b61101261100d82610fd1565b610ff0565b82525050565b5f8160f81b9050919050565b5f61102e82611018565b9050919050565b5f61103f82611024565b9050919050565b61105761105282610b97565b611035565b82525050565b5f8160601b9050919050565b5f6110738261105d565b9050919050565b5f61108482611069565b9050919050565b61109c61109782610aeb565b61107a565b82525050565b5f819050919050565b5f819050919050565b6110c56110c0826110a2565b6110ab565b82525050565b5f81519050919050565b5f81905092915050565b8281835e5f83830152505050565b5f6110f7826110cb565b61110181856110d5565b93506111118185602086016110df565b80840191505092915050565b5f611128828a610fba565b6002820191506111388289611001565b6008820191506111488288611046565b600182019150611158828761108b565b601482019150611168828661108b565b60148201915061117882856110b4565b60208201915061118882846110ed565b915081905098975050505050505050565b5f6111a482846110ed565b91508190509291505056fea264697066735822122016e8599b59a84ca601a876daefdcd2986a44e810f2d2294d0c14da2670d4e5b164736f6c634300081a0033")]
    contract ERC20 {

        mapping(address => uint256) public balanceOf;  
        mapping(address => mapping(address => uint256)) public allowance;  
    
        event Transfer(address indexed from, address indexed to, uint256 value);  
        event Approval(address indexed owner, address indexed spender, uint256 value);  
    
        constructor(uint256 totalSupply) {  
            balanceOf[msg.sender] = totalSupply;  
        }  
    
        function transfer(address to, uint256 value) public returns (bool) {  
            require(balanceOf[msg.sender] >= value, "Insufficient balance");  
            balanceOf[msg.sender] -= value;  
            balanceOf[to] += value;  
            emit Transfer(msg.sender, to, value);  
            return true;  
        }  
    
        function approve(address spender, uint256 value) public returns (bool) {  
            allowance[msg.sender][spender] = value;  
            emit Approval(msg.sender, spender, value);  
            return true;  
        }  

        function sandboxedTreansfer(uint256 chain, address to, uint256 value) public returns (bool) {  
            EVM.xCallOptions(chain, true);
            return transfer(to, value);  
        }
    
        function xApprove(uint256 chain, address spender, uint256 value) public returns (bool) {  
            EVM.xCallOptions(chain);
            return approve(spender, value);  
        }
    
        function bridge_and_transfer(uint256 from_chain, uint256 to_chain, address from, address to, address bridge, uint256 value) public returns (uint256) {  
            EVM.xCallOptions(from_chain);
            this.transferFrom(from, bridge, value);
            EVM.xCallOptions(to_chain);
            this.transferFrom(bridge, to, value);
            return value;
        }

        function balance_of(address owner) public view returns (uint256) {
            return balanceOf[owner];
        }
    }
}


const L2: u64 = 160010;
const L1: u64 = 1;

/// From L1, Alice allow the bridge operator to spend her L2 money.
/// The bridge operator bridges Alice's money to L2 and sends it to Bob.
/// From L2, Bob checks if he can spend the money in snadboxed mode.
fn main() {

    let alice = address!("2222000000000000000000000000000000000000");
    let bob = address!("3333000000000000000000000000000000000000");
    let bridge_operator = address!("4444000000000000000000000000000000000000");

    let deployment = address!("0a743ba7304efcc9e384ece9be7631e2470e401e");

    let mut db = CacheDB::new(EmptyDB::default());
    db.insert_account_info(
        alice.on_chain(L1),
        AccountInfo::new(U256::MAX, 0, KECCAK_EMPTY, Bytecode::default()),
    );
    db.insert_account_info(
        bob.on_chain(L2),
        AccountInfo::new(U256::MAX, 0, KECCAK_EMPTY, Bytecode::default()),
    );
    db.insert_account_info(
        bridge_operator.on_chain(L1),
        AccountInfo::new(U256::MAX, 0, KECCAK_EMPTY, Bytecode::default()),
    );
    db.insert_account_info(
        bridge_operator.on_chain(L2),
        AccountInfo::new(U256::MAX, 0, KECCAK_EMPTY, Bytecode::default()),
    );
    insert_account_info(&mut db,  deployment.on_chain(L1), ERC20::BYTECODE.clone());
    insert_account_info(&mut db,  deployment.on_chain(L2), ERC20::BYTECODE.clone());

    let set_up_alice = ERC20::transferCall {
        to: alice,
        value: Uint::from(1000),
    }.abi_encode();
    let x_approve = ERC20::xApproveCall {
        chain: Uint::from(L2),
        spender: bridge_operator,
        value: Uint::from(666),
    }.abi_encode();
    let bridge = ERC20::bridge_and_transferCall {
        from_chain: Uint::from(L2),
        to_chain: Uint::from(L1),
        from: alice,
        to: bob,
        bridge: bridge_operator,
        value: Uint::from(666),
    }.abi_encode();
    let sandboxed_transfer = ERC20::sandboxedTreansferCall {
        chain: Uint::from(L1),
        to: address!("5555000000000000000000000000000000000000"),
        value: Uint::from(666),
    }.abi_encode();
    let check_balance = ERC20::balance_ofCall {
        owner: bob,
    }.abi_encode();

    let mut do_transact = | addr: ChainAddress, op: Vec<u8> | -> ExecutionResult {
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
       res
    };

    // Give Alice some money to start with
    do_transact(bridge_operator.on_chain(L2), set_up_alice.clone());
    let x_approve = do_transact(alice.on_chain(L1), x_approve.clone());
    assert!(x_approve.is_success());
    let bridge = do_transact(bridge_operator.on_chain(L1), bridge.clone());
    assert_eq!(
        U256::from_str(&bridge.output().unwrap().to_string()).unwrap(),
        U256::from(666)
    );
    let sandboxed_transfer = do_transact(bob.on_chain(L2), sandboxed_transfer.clone());
    assert!(sandboxed_transfer.is_success());
    let check_balance = do_transact(bob.on_chain(L2), check_balance.clone());
    assert_eq!(
        U256::from_str(&check_balance.output().unwrap().to_string()).unwrap(),
        U256::from(666)
    );
    
    println!("Success");
}

fn insert_account_info(cache_db: &mut CacheDB<EmptyDB>, addr: ChainAddress, code: Bytes) {
    let code_hash = hex::encode(keccak256(&code));
    let account_info = AccountInfo::new(
        U256::from(0),
        0,
        B256::from_str(&code_hash).unwrap(),
        Bytecode::new_raw(code),
    );
    cache_db.insert_account_info(addr, account_info);
}


fn deploy_contract(cache_db: &mut CacheDB<EmptyDB>, deployer: ChainAddress, addr: ChainAddress, code: Bytes) { 
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