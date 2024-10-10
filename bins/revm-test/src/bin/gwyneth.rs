use std::str::FromStr;

use alloy_sol_macro::sol;
use alloy_sol_types::{sol_data::Address, SolCall, SolInterface};
use revm::{db::{CacheDB, EmptyDB}, primitives::{address, keccak256, ruint::Uint, AccountInfo, Bytecode, Bytes, ChainAddress, ExecutionResult, Output, TransactTo, B256, KECCAK_EMPTY, U256}, Evm};

sol!{
    
    #[sol(bytecode="0x608060405234801561000f575f80fd5b506004361061003f575f3560e01c806366fd8cff14610043578063893d20e814610061578063a6f9dae11461007f575b5f80fd5b61004b61009b565b604051610058919061039f565b60405180910390f35b610069610116565b604051610076919061039f565b60405180910390f35b610099600480360381019061009491906103e6565b61013d565b005b5f6100a4610266565b3073ffffffffffffffffffffffffffffffffffffffff1663893d20e86040518163ffffffff1660e01b8152600401602060405180830381865afa1580156100ed573d5f803e3d5ffd5b505050506040513d601f19601f820116820180604052508101906101119190610425565b905090565b5f805f9054906101000a900473ffffffffffffffffffffffffffffffffffffffff16905090565b5f73ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff16036101ab576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016101a2906104d0565b60405180910390fd5b8073ffffffffffffffffffffffffffffffffffffffff165f8054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff167f342827c97908e5e2f71151c08502a66d44b6f758e3ac2f1de95f02eb95f0a73560405160405180910390a3805f806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff16021790555050565b6102706001610272565b565b61027d816001610280565b50565b61028c82825f80610290565b5050565b6102ae848484845f801b60405180602001604052805f8152506102b4565b50505050565b5f60018787878787876040516020016102d39796959493929190610686565b60405160208183030381529060405290505f6104d273ffffffffffffffffffffffffffffffffffffffff168260405161030c9190610702565b5f60405180830381855afa9150503d805f8114610344576040519150601f19603f3d011682016040523d82523d5f602084013e610349565b606091505b5050905080610356575f80fd5b5050505050505050565b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f61038982610360565b9050919050565b6103998161037f565b82525050565b5f6020820190506103b25f830184610390565b92915050565b5f80fd5b6103c58161037f565b81146103cf575f80fd5b50565b5f813590506103e0816103bc565b92915050565b5f602082840312156103fb576103fa6103b8565b5b5f610408848285016103d2565b91505092915050565b5f8151905061041f816103bc565b92915050565b5f6020828403121561043a576104396103b8565b5b5f61044784828501610411565b91505092915050565b5f82825260208201905092915050565b7f4e6577206f776e65722073686f756c64206e6f7420626520746865207a65726f5f8201527f2061646472657373000000000000000000000000000000000000000000000000602082015250565b5f6104ba602883610450565b91506104c582610460565b604082019050919050565b5f6020820190508181035f8301526104e7816104ae565b9050919050565b5f61ffff82169050919050565b5f8160f01b9050919050565b5f610511826104fb565b9050919050565b610529610524826104ee565b610507565b82525050565b5f67ffffffffffffffff82169050919050565b5f8160c01b9050919050565b5f61055882610542565b9050919050565b61057061056b8261052f565b61054e565b82525050565b5f8115159050919050565b5f8160f81b9050919050565b5f61059782610581565b9050919050565b5f6105a88261058d565b9050919050565b6105c06105bb82610576565b61059e565b82525050565b5f8160601b9050919050565b5f6105dc826105c6565b9050919050565b5f6105ed826105d2565b9050919050565b6106056106008261037f565b6105e3565b82525050565b5f819050919050565b5f819050919050565b61062e6106298261060b565b610614565b82525050565b5f81519050919050565b5f81905092915050565b8281835e5f83830152505050565b5f61066082610634565b61066a818561063e565b935061067a818560208601610648565b80840191505092915050565b5f610691828a610518565b6002820191506106a1828961055f565b6008820191506106b182886105af565b6001820191506106c182876105f4565b6014820191506106d182866105f4565b6014820191506106e1828561061d565b6020820191506106f18284610656565b915081905098975050505050505050565b5f61070d8284610656565b91508190509291505056fea2646970667358221220a21eb21a3c3001706194aacb629a456a37302eeb74895e37cec72bcb499ffa3564736f6c634300081a0033")]
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
    
        function getOwnerL1() external view returns (address) {
            EVM.xCallOnL1();
            return this.getOwner();
        }
    
    }
}

const L2: u64 = 160010;
const L1: u64 = 1;

fn main() {
    let myself1 = ChainAddress(L1, address!("1000000000000000000000000000000000000000"));
    let myself2 = ChainAddress(L2, address!("1000000000000000000000000000000000000000"));

    let deploy = address!("0a743ba7304efcc9e384ece9be7631e2470e401e");
    // let l2_deploy = address!("49206861766520746f6f206d7563682074696d65");
    
    let mut db = CacheDB::new(EmptyDB::default());
    insert_account_info(
        &mut db, 
        ChainAddress(L2, deploy), 
        Owner::BYTECODE.clone()
    );
    insert_account_info(
        &mut db, 
        ChainAddress(L1, deploy), 
        Owner::BYTECODE.clone()
    );
    db.insert_account_info(
        myself1, 
        AccountInfo::new(
            U256::MAX,
            0,
            KECCAK_EMPTY,
            Bytecode::default()
        )
    );
    db.insert_account_info(
        myself2, 
        AccountInfo::new(
            U256::MAX,
            0,
            KECCAK_EMPTY,
            Bytecode::default()
        )
    );

    let call_1 = Owner::changeOwnerCall { newOwner: myself1.1 }.abi_encode();
    let call_2 = Owner::getOwnerL1Call { }.abi_encode();

    let mut evm = Evm::builder()
        .modify_tx_env(|tx| {
            tx.caller = myself1;
            tx.transact_to = TransactTo::Call(ChainAddress(L1, deploy));
            tx.data = call_1.clone().into();
        })
        .with_db(&mut db)
        .build();

    let result_1 = evm.transact().unwrap().result;
    println!("Set owner on L1 {:?} \n\n\n\n", result_1);
    drop(evm);

    let mut evm = Evm::builder()
        .modify_tx_env(|tx| {
            tx.caller = myself2;
            tx.transact_to = TransactTo::Call(ChainAddress(L2, deploy));
            tx.data = call_2.clone().into();
        })
        .with_db(&mut db)
        .build();

    let result_2 = evm.transact().unwrap().result;
    println!("Read owner from L2 {:?}", result_2);

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