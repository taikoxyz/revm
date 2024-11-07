use std::{io::Read, str::FromStr};

use alloy_sol_macro::sol;
use alloy_sol_types::{sol_data::Address, SolCall, SolInterface, SolType};
use revm::{
    db::{CacheDB, EmptyDB}, primitives::{
        address, keccak256, ruint::Uint, AccountInfo, Bytecode, Bytes, ChainAddress, ExecutionResult, OnChain, Output, TransactTo, B256, KECCAK_EMPTY, U256
    }, Database, Evm
};


sol! {
    #[sol(bytecode="608060405234801561000f575f5ffd5b506004361061004a575f3560e01c8063088a25ca1461004e57806366fd8cff1461006a578063893d20e814610088578063a6f9dae1146100a6575b5f5ffd5b61006860048036038101906100639190610457565b6100c2565b005b610072610133565b60405161007f9190610491565b60405180910390f35b6100906101ae565b60405161009d9190610491565b60405180910390f35b6100c060048036038101906100bb9190610457565b6101d5565b005b6100ca6102ff565b3073ffffffffffffffffffffffffffffffffffffffff1663a6f9dae1826040518263ffffffff1660e01b81526004016101039190610491565b5f604051808303815f87803b15801561011a575f5ffd5b505af115801561012c573d5f5f3e3d5ffd5b5050505050565b5f61013c6102ff565b3073ffffffffffffffffffffffffffffffffffffffff1663893d20e86040518163ffffffff1660e01b8152600401602060405180830381865afa158015610185573d5f5f3e3d5ffd5b505050506040513d601f19601f820116820180604052508101906101a991906104be565b905090565b5f5f5f9054906101000a900473ffffffffffffffffffffffffffffffffffffffff16905090565b5f73ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1603610243576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161023a90610569565b60405180910390fd5b8073ffffffffffffffffffffffffffffffffffffffff165f5f9054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff167f342827c97908e5e2f71151c08502a66d44b6f758e3ac2f1de95f02eb95f0a73560405160405180910390a3805f5f6101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff16021790555050565b610309600161030b565b565b610316816001610319565b50565b61032582825f5f610329565b5050565b610347848484845f5f1b60405180602001604052805f81525061034d565b50505050565b5f600187878787878760405160200161036c979695949392919061071f565b60405160208183030381529060405290505f6104d273ffffffffffffffffffffffffffffffffffffffff16826040516103a5919061079b565b5f60405180830381855afa9150503d805f81146103dd576040519150601f19603f3d011682016040523d82523d5f602084013e6103e2565b606091505b50509050806103ef575f5ffd5b5050505050505050565b5f5ffd5b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f610426826103fd565b9050919050565b6104368161041c565b8114610440575f5ffd5b50565b5f813590506104518161042d565b92915050565b5f6020828403121561046c5761046b6103f9565b5b5f61047984828501610443565b91505092915050565b61048b8161041c565b82525050565b5f6020820190506104a45f830184610482565b92915050565b5f815190506104b88161042d565b92915050565b5f602082840312156104d3576104d26103f9565b5b5f6104e0848285016104aa565b91505092915050565b5f82825260208201905092915050565b7f4e6577206f776e65722073686f756c64206e6f7420626520746865207a65726f5f8201527f2061646472657373000000000000000000000000000000000000000000000000602082015250565b5f6105536028836104e9565b915061055e826104f9565b604082019050919050565b5f6020820190508181035f83015261058081610547565b9050919050565b5f61ffff82169050919050565b5f8160f01b9050919050565b5f6105aa82610594565b9050919050565b6105c26105bd82610587565b6105a0565b82525050565b5f67ffffffffffffffff82169050919050565b5f8160c01b9050919050565b5f6105f1826105db565b9050919050565b610609610604826105c8565b6105e7565b82525050565b5f8115159050919050565b5f8160f81b9050919050565b5f6106308261061a565b9050919050565b5f61064182610626565b9050919050565b6106596106548261060f565b610637565b82525050565b5f8160601b9050919050565b5f6106758261065f565b9050919050565b5f6106868261066b565b9050919050565b61069e6106998261041c565b61067c565b82525050565b5f819050919050565b5f819050919050565b6106c76106c2826106a4565b6106ad565b82525050565b5f81519050919050565b5f81905092915050565b8281835e5f83830152505050565b5f6106f9826106cd565b61070381856106d7565b93506107138185602086016106e1565b80840191505092915050565b5f61072a828a6105b1565b60028201915061073a82896105f8565b60088201915061074a8288610648565b60018201915061075a828761068d565b60148201915061076a828661068d565b60148201915061077a82856106b6565b60208201915061078a82846106ef565b915081905098975050505050505050565b5f6107a682846106ef565b91508190509291505056fea2646970667358221220e1c66ba0fc97da09855ae49b3097a51844279d93f515534340b0782d3418de6864736f6c634300081c0033")]
    contract Owner {

        address private owner;
    
    
        // event for EVM logging
        event OwnerSet(address indexed oldOwner, address indexed newOwner);
    
        /**
         * @dev Change owner
         * @param newOwner address of new owner
         */
        function changeOwner(address newOwner) public {
            require(newOwner != address(0), "New owner should not be the zero address");
            emit OwnerSet(owner, newOwner);
            owner = newOwner;
        }
    
        function changeOwnerL1(address newOwner) public {
            EVM.xCallOnL1();
            this.changeOwner(newOwner);
        }
    
    
        /**
         * @dev Return owner address 
         * @return address of owner
         */
        function getOwner() external view returns (address) {
            return owner;
        }
    
        function getOwnerL1() external view returns (address) {
            EVM.xCallOnL1();
            return this.getOwner();
        }
    
    }
}


const A: u64 = 1;
const B: u64 = 160010;

/// This example sets L1 owner from L1 and reads it from L2.
/// Then sets a new L1 owner from L2 and reads it from L1.
fn main() {

    let owner = address!("2222000000000000000000000000000000000000");
    let new_owner = address!("3333000000000000000000000000000000000000");

    
    let a_user = ChainAddress(A, address!("1000000000000000000000000000000000000000"));
    let b_user = ChainAddress(B, address!("1000000000000000000000000000000000000000"));

    let deployment = address!("0a743ba7304efcc9e384ece9be7631e2470e401e");

    let mut db = CacheDB::new(EmptyDB::default());
    db.insert_account_info(
        a_user,
        AccountInfo::new(U256::MAX, 0, KECCAK_EMPTY, Bytecode::default()),
    );
    db.insert_account_info(
        b_user,
        AccountInfo::new(U256::MAX, 0, KECCAK_EMPTY, Bytecode::default()),
    );
    insert_account_info(&mut db, deployment.on_chain(B), Owner::BYTECODE.clone());
    insert_account_info(&mut db, deployment.on_chain(A), Owner::BYTECODE.clone());


    let set_native = Owner::changeOwnerCall { newOwner: owner }.abi_encode();
    let set_x = Owner::changeOwnerL1Call { newOwner: new_owner }.abi_encode();
    let get_native = Owner::getOwnerCall {}.abi_encode();
    let get_x = Owner::getOwnerL1Call {}.abi_encode();

    let mut do_transact = | addr: ChainAddress, op: Vec<u8> | -> ExecutionResult {
        println!("\n\n");
        let mut evm = Evm::builder()
            .modify_tx_env(|tx| {
                tx.caller = addr;
                tx.transact_to = TransactTo::Call(deployment.on_chain(addr.0));
                tx.data = op.clone().into();
            })
            .with_db(&mut db)
            .build();
       let res = evm.transact().unwrap().result;
       evm.transact_commit().unwrap();
       drop(evm);
       res
    };

    // Set L1 owner from L1 and read it from L2
    do_transact(a_user, set_native.clone());
    let l2_read = do_transact(b_user, get_x.clone());
    assert_eq!(
        owner.to_string(),
        Address::abi_decode(l2_read.output().unwrap(), false).unwrap().to_string()
    );

    // Set L1 owner from L2 and read it from L1
    do_transact(b_user, set_x.clone());
    let l1_read = do_transact(a_user, get_native.clone());
    assert_eq!(
        new_owner.to_string(),
        Address::abi_decode(l1_read.output().unwrap(), false).unwrap().to_string()
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
