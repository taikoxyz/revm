use std::str::FromStr;

use alloy_sol_macro::sol;
use alloy_sol_types::{sol_data::Address, SolCall, SolInterface};
use revm::{db::{CacheDB, EmptyDB}, interpreter::analysis::to_analysed, primitives::{address, keccak256, ruint::Uint, AccountInfo, Bytecode, Bytes, ChainAddress, ExecutionResult, Output, TransactTo, B256, KECCAK_EMPTY, U256}, Evm};
// use Counter::{getNumberCall, setNumberCall, CounterCalls};
use Owner::{changeOwnerCall, constructorCall, getOwnerCall};


sol! {
    // replaced EVM contract placeholder with the hardedcoded address
    #[sol(bytecode="608060405234801561000f575f80fd5b506100556040518060400160405280601b81526020017f4f776e657220636f6e7472616374206465706c6f7965642062793a00000000008152503361011260201b60201c565b335f806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055505f8054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff165f73ffffffffffffffffffffffffffffffffffffffff167f342827c97908e5e2f71151c08502a66d44b6f758e3ac2f1de95f02eb95f0a73560405160405180910390a361031d565b6101b082826040516024016101289291906102c2565b6040516020818303038152906040527f319af333000000000000000000000000000000000000000000000000000000007bffffffffffffffffffffffffffffffffffffffffffffffffffffffff19166020820180517bffffffffffffffffffffffffffffffffffffffffffffffffffffffff83818316178352505050506101b460201b60201c565b5050565b6101d5816101d06101d860201b61024f176101f760201b60201c565b60201c565b50565b5f6a636f6e736f6c652e6c6f6790505f80835160208501845afa505050565b61020960201b61026e17819050919050565b6102116102f0565b565b5f81519050919050565b5f82825260208201905092915050565b8281835e5f83830152505050565b5f601f19601f8301169050919050565b5f61025582610213565b61025f818561021d565b935061026f81856020860161022d565b6102788161023b565b840191505092915050565b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f6102ac82610283565b9050919050565b6102bc816102a2565b82525050565b5f6040820190508181035f8301526102da818561024b565b90506102e960208301846102b3565b9392505050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52605160045260245ffd5b6104928061032a5f395ff3fe608060405234801561000f575f80fd5b5060043610610034575f3560e01c8063893d20e814610038578063a6f9dae114610056575b5f80fd5b610040610072565b60405161004d91906102b7565b60405180910390f35b610070600480360381019061006b91906102fe565b610099565b005b5f805f9054906101000a900473ffffffffffffffffffffffffffffffffffffffff16905090565b5f8054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff1614610126576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161011d90610383565b60405180910390fd5b5f73ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1603610194576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161018b90610411565b60405180910390fd5b8073ffffffffffffffffffffffffffffffffffffffff165f8054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff167f342827c97908e5e2f71151c08502a66d44b6f758e3ac2f1de95f02eb95f0a73560405160405180910390a3805f806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff16021790555050565b5f6a636f6e736f6c652e6c6f6790505f80835160208501845afa505050565b61027661042f565b565b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f6102a182610278565b9050919050565b6102b181610297565b82525050565b5f6020820190506102ca5f8301846102a8565b92915050565b5f80fd5b6102dd81610297565b81146102e7575f80fd5b50565b5f813590506102f8816102d4565b92915050565b5f60208284031215610313576103126102d0565b5b5f610320848285016102ea565b91505092915050565b5f82825260208201905092915050565b7f43616c6c6572206973206e6f74206f776e6572000000000000000000000000005f82015250565b5f61036d601383610329565b915061037882610339565b602082019050919050565b5f6020820190508181035f83015261039a81610361565b9050919050565b7f4e6577206f776e65722073686f756c64206e6f7420626520746865207a65726f5f8201527f2061646472657373000000000000000000000000000000000000000000000000602082015250565b5f6103fb602883610329565b9150610406826103a1565b604082019050919050565b5f6020820190508181035f830152610428816103ef565b9050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52605160045260245ffdfea2646970667358221220fd695c5ac7e26c747dc5d69109c3f6021d76af5ca3ad6ba5c12b131b151355b964736f6c634300081a0033")]
    contract Owner {

        address private owner;
    
        // event for EVM logging
        event OwnerSet(address indexed oldOwner, address indexed newOwner);
    
        // modifier to check if caller is owner
        modifier isOwner() {
            // If the first argument of 'require' evaluates to 'false', execution terminates and all
            // changes to the state and to Ether balances are reverted.
            // This used to consume all gas in old EVM versions, but not anymore.
            // It is often a good idea to use 'require' to check if functions are called correctly.
            // As a second argument, you can also provide an explanation about what went wrong.
            require(msg.sender == owner, "Caller is not owner");
            _;
        }
    
        /**
         * @dev Set contract deployer as owner
         */
        constructor() {
            console.log("Owner contract deployed by:", msg.sender);
            owner = msg.sender; // 'msg.sender' is sender of current call, contract deployer for a constructor
            emit OwnerSet(address(0), owner);
        }
    
        /**
         * @dev Change owner
         * @param newOwner address of new owner
         */
        function changeOwner(address newOwner) public isOwner {
            require(newOwner != address(0), "New owner should not be the zero address");
            emit OwnerSet(owner, newOwner);
            owner = newOwner;
        }
    
        /**
         * @dev Return owner address 
         * @return address of owner
         */
        function getOwner() external view returns (address) {
            return owner;
        }
    } 
    
}

const L2: u64 = 160010;
const L1: u64 = 1;


fn main() {

    let myself = ChainAddress(L1, address!("1000000000000000000000000000000000000000"));
    let counter_deployment = address!("49206861766520746f6f206d7563682074696d65");
    let bytecode = to_analysed(Bytecode::new_raw(Owner::BYTECODE.clone()));
    
    let mut db = CacheDB::new(EmptyDB::default());
    insert_account_info(
        &mut db, 
        ChainAddress(L1, counter_deployment), 
        bytecode.clone().bytes()
    );
    db.insert_account_info(
        ChainAddress(L1, address!("1000000000000000000000000000000000000000")), 
        AccountInfo::new(
            U256::MAX,
            0,
            KECCAK_EMPTY,
            Bytecode::default()
        )
    );

    let call_1 = changeOwnerCall { newOwner: myself.1 }.abi_encode();
    let call_2 = getOwnerCall { }.abi_encode();

    // println!("Call 1: {:?}", call_1);

    // let mut evm = Evm::builder()
    //     .modify_tx_env(|tx| {
    //         tx.caller = myself;
    //         tx.transact_to = TransactTo::Call(ChainAddress(L1, counter_deployment));
    //         tx.data = call_1.clone().into();
    //     })
    //     .with_db(&mut db)
    //     .build();

    // let result_1 = evm.transact().unwrap();
    // drop(evm);
    // println!("Set number on L1 {:?}", result_1);

    let mut evm = Evm::builder()
        .modify_tx_env(|tx| {
            tx.caller = myself;
            tx.transact_to = TransactTo::Call(ChainAddress(L1, counter_deployment));
            tx.data = call_2.clone().into();
        })
        .with_db(&mut db)
        .build();

    let result_2 = evm.transact().unwrap();
    drop(evm);
    println!("Get number on L1 {:?}", result_2);


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