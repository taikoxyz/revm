use std::{io::Read, str::FromStr};

use alloy_sol_macro::sol;
use alloy_sol_types::{sol_data::Address, SolCall, SolInterface, SolType};
use revm::{
    db::{CacheDB, EmptyDB},
    primitives::{
        address, create_state_diff, keccak256, ruint::Uint, AccountInfo, Bytecode, Bytes, ChainAddress, ExecutionResult, OnChain, Output, TransactTo, B256, KECCAK_EMPTY, U256
    },
    Database, Evm,
};

sol! {
    // Compiled with constructor parameter = 99999
    #[sol(bytecode="6080604052348015600e575f80fd5b506201869f5f803373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f2081905550611c0e806100605f395ff3fe6080604052600436106100c1575f3560e01c806370a082311161007e578063c664c71411610058578063c664c714146102e1578063cb05d55114610311578063d48024a714610341578063dd62ed3e1461037d576100c1565b806370a082311461022d578063a9059cbb14610269578063b3f322af146102a5576100c1565b8063095ea7b3146100c5578063104e81ff1461010157806323b872dd1461013d578063297926631461017957806330e0789e146101b55780634e6ec247146101f1575b5f80fd5b3480156100d0575f80fd5b506100eb60048036038101906100e691906112b9565b6103b9565b6040516100f89190611306565b60405180910390f35b34801561010c575f80fd5b506101276004803603810190610122919061131f565b6104a5565b6040516101349190611306565b60405180910390f35b348015610148575f80fd5b50610163600480360381019061015e919061131f565b610600565b6040516101709190611306565b60405180910390f35b348015610184575f80fd5b5061019f600480360381019061019a919061136f565b610944565b6040516101ac9190611306565b60405180910390f35b3480156101c0575f80fd5b506101db60048036038101906101d6919061131f565b6109d5565b6040516101e89190611306565b60405180910390f35b3480156101fc575f80fd5b50610217600480360381019061021291906112b9565b610af2565b6040516102249190611306565b60405180910390f35b348015610238575f80fd5b50610253600480360381019061024e91906113bf565b610bbc565b6040516102609190611306565b60405180910390f35b348015610274575f80fd5b5061028f600480360381019061028a91906112b9565b610bd0565b60405161029c9190611306565b60405180910390f35b3480156102b0575f80fd5b506102cb60048036038101906102c6919061136f565b610d62565b6040516102d89190611306565b60405180910390f35b6102fb60048036038101906102f69190611425565b610df5565b604051610308919061147d565b60405180910390f35b61032b60048036038101906103269190611496565b610e99565b604051610338919061147d565b60405180910390f35b34801561034c575f80fd5b506103676004803603810190610362919061136f565b610f28565b6040516103749190611306565b60405180910390f35b348015610388575f80fd5b506103a3600480360381019061039e91906114e6565b611009565b6040516103b09190611306565b60405180910390f35b5f8160015f3373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20819055508273ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925846040516104949190611306565b60405180910390a381905092915050565b5f3073ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff1614610514576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161050b906115a4565b60405180910390fd5b8160015f8673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20819055508273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925846040516105ee9190611306565b60405180910390a38190509392505050565b5f815f808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20541015610680576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016106779061160c565b60405180910390fd5b3373ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff161461076f578160015f8673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f3373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f2054101561076e576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161076590611674565b60405180910390fd5b5b815f808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8282546107ba91906116bf565b92505081905550815f808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f82825461080c91906116f2565b925050819055503373ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff16146108d5578160015f8673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f3373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8282546108cd91906116bf565b925050819055505b8273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef846040516109329190611306565b60405180910390a38190509392505050565b5f61094e84611029565b503073ffffffffffffffffffffffffffffffffffffffff1663104e81ff3385856040518463ffffffff1660e01b815260040161098c93929190611734565b6020604051808303815f875af11580156109a8573d5f803e3d5ffd5b505050506040513d601f19601f820116820180604052508101906109cc919061177d565b90509392505050565b5f3073ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff1614610a44576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610a3b906117f2565b60405180910390fd5b815f808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f828254610a8f91906116bf565b92505081905550815f808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f828254610ae191906116f2565b925050819055508190509392505050565b5f3073ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff1614610b61576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610b58906117f2565b60405180910390fd5b815f808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f828254610bac91906116f2565b9250508190555081905092915050565b5f602052805f5260405f205f915090505481565b5f815f803373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20541015610c50576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610c479061160c565b60405180910390fd5b815f803373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f828254610c9b91906116bf565b92505081905550815f808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f828254610ced91906116f2565b925050819055508273ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef84604051610d519190611306565b60405180910390a381905092915050565b5f610d6e84600161103b565b503073ffffffffffffffffffffffffffffffffffffffff166330e0789e3385856040518463ffffffff1660e01b8152600401610dac93929190611734565b6020604051808303815f875af1158015610dc8573d5f803e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610dec919061177d565b90509392505050565b5f80610e20848473ffffffffffffffffffffffffffffffffffffffff1661105090919063ffffffff16565b73ffffffffffffffffffffffffffffffffffffffff1634604051610e439061183d565b5f6040518083038185875af1925050503d805f8114610e7d576040519150601f19603f3d011682016040523d82523d5f602084013e610e82565b606091505b5050905080610e8f575f80fd5b8091505092915050565b5f610ea38461108c565b73ffffffffffffffffffffffffffffffffffffffff1663c664c7143485856040518463ffffffff1660e01b8152600401610ede929190611860565b60206040518083038185885af1158015610efa573d5f803e3d5ffd5b50505050506040513d601f19601f82011682018060405250810190610f1f91906118b1565b90509392505050565b5f815f803373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f828254610f7491906116bf565b92505081905550610f8484611029565b503073ffffffffffffffffffffffffffffffffffffffff16634e6ec24784846040518363ffffffff1660e01b8152600401610fc09291906118dc565b6020604051808303815f875af1158015610fdc573d5f803e3d5ffd5b505050506040513d601f19601f82011682018060405250810190611000919061177d565b90509392505050565b6001602052815f5260405f20602052805f5260405f205f91509150505481565b5f611034825f61103b565b9050919050565b5f6110488383323061109e565b905092915050565b5f8061105c835f61103b565b9050801561106d5783915050611086565b731adb9959eb142be128e6dfecc8d571f07cd66dee9150505b92915050565b5f61109782306110c7565b9050919050565b5f6110bd858585855f801b60405180602001604052805f8152506110f9565b9050949350505050565b5f6110f1838373ffffffffffffffffffffffffffffffffffffffff1661105090919063ffffffff16565b905092915050565b5f46870361110a576001905061121e565b5f60018888888888886040516020016111299796959493929190611a86565b60405160208183030381529060405290505f806104d273ffffffffffffffffffffffffffffffffffffffff16836040516111639190611b02565b5f60405180830381855afa9150503d805f811461119b576040519150601f19603f3d011682016040523d82523d5f602084013e6111a0565b606091505b509150915081801561121857507f6c5413304f1a20a2eeffee31f1dcf2ed47473a68199e6844276e42732fc71b8b7bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916816111f790611b72565b7bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916145b93505050505b9695505050505050565b5f80fd5b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f6112558261122c565b9050919050565b6112658161124b565b811461126f575f80fd5b50565b5f813590506112808161125c565b92915050565b5f819050919050565b61129881611286565b81146112a2575f80fd5b50565b5f813590506112b38161128f565b92915050565b5f80604083850312156112cf576112ce611228565b5b5f6112dc85828601611272565b92505060206112ed858286016112a5565b9150509250929050565b61130081611286565b82525050565b5f6020820190506113195f8301846112f7565b92915050565b5f805f6060848603121561133657611335611228565b5b5f61134386828701611272565b935050602061135486828701611272565b9250506040611365868287016112a5565b9150509250925092565b5f805f6060848603121561138657611385611228565b5b5f611393868287016112a5565b93505060206113a486828701611272565b92505060406113b5868287016112a5565b9150509250925092565b5f602082840312156113d4576113d3611228565b5b5f6113e184828501611272565b91505092915050565b5f6113f48261122c565b9050919050565b611404816113ea565b811461140e575f80fd5b50565b5f8135905061141f816113fb565b92915050565b5f806040838503121561143b5761143a611228565b5b5f611448858286016112a5565b925050602061145985828601611411565b9150509250929050565b5f8115159050919050565b61147781611463565b82525050565b5f6020820190506114905f83018461146e565b92915050565b5f805f606084860312156114ad576114ac611228565b5b5f6114ba868287016112a5565b93505060206114cb868287016112a5565b92505060406114dc86828701611411565b9150509250925092565b5f80604083850312156114fc576114fb611228565b5b5f61150985828601611272565b925050602061151a85828601611272565b9150509250929050565b5f82825260208201905092915050565b7f4f6e6c7920636f6e747261637420697473656c662063616e2063616c6c2074685f8201527f69732066756e6374696f6e000000000000000000000000000000000000000000602082015250565b5f61158e602b83611524565b915061159982611534565b604082019050919050565b5f6020820190508181035f8301526115bb81611582565b9050919050565b7f496e73756666696369656e742062616c616e63650000000000000000000000005f82015250565b5f6115f6601483611524565b9150611601826115c2565b602082019050919050565b5f6020820190508181035f830152611623816115ea565b9050919050565b7f416c6c6f77616e636520657863656564656400000000000000000000000000005f82015250565b5f61165e601283611524565b91506116698261162a565b602082019050919050565b5f6020820190508181035f83015261168b81611652565b9050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b5f6116c982611286565b91506116d483611286565b92508282039050818111156116ec576116eb611692565b5b92915050565b5f6116fc82611286565b915061170783611286565b925082820190508082111561171f5761171e611692565b5b92915050565b61172e8161124b565b82525050565b5f6060820190506117475f830186611725565b6117546020830185611725565b61176160408301846112f7565b949350505050565b5f815190506117778161128f565b92915050565b5f6020828403121561179257611791611228565b5b5f61179f84828501611769565b91505092915050565b7f4f6e6c79207468697320636f6e74726163742063616e206d696e7400000000005f82015250565b5f6117dc601b83611524565b91506117e7826117a8565b602082019050919050565b5f6020820190508181035f830152611809816117d0565b9050919050565b5f81905092915050565b50565b5f6118285f83611810565b91506118338261181a565b5f82019050919050565b5f6118478261181d565b9150819050919050565b61185a816113ea565b82525050565b5f6040820190506118735f8301856112f7565b6118806020830184611851565b9392505050565b61189081611463565b811461189a575f80fd5b50565b5f815190506118ab81611887565b92915050565b5f602082840312156118c6576118c5611228565b5b5f6118d38482850161189d565b91505092915050565b5f6040820190506118ef5f830185611725565b6118fc60208301846112f7565b9392505050565b5f61ffff82169050919050565b5f8160f01b9050919050565b5f61192682611910565b9050919050565b61193e61193982611903565b61191c565b82525050565b5f67ffffffffffffffff82169050919050565b5f8160c01b9050919050565b5f61196d82611957565b9050919050565b61198561198082611944565b611963565b82525050565b5f8160f81b9050919050565b5f6119a18261198b565b9050919050565b5f6119b282611997565b9050919050565b6119ca6119c582611463565b6119a8565b82525050565b5f8160601b9050919050565b5f6119e6826119d0565b9050919050565b5f6119f7826119dc565b9050919050565b611a0f611a0a8261124b565b6119ed565b82525050565b5f819050919050565b5f819050919050565b611a38611a3382611a15565b611a1e565b82525050565b5f81519050919050565b8281835e5f83830152505050565b5f611a6082611a3e565b611a6a8185611810565b9350611a7a818560208601611a48565b80840191505092915050565b5f611a91828a61192d565b600282019150611aa18289611974565b600882019150611ab182886119b9565b600182019150611ac182876119fe565b601482019150611ad182866119fe565b601482019150611ae18285611a27565b602082019150611af18284611a56565b915081905098975050505050505050565b5f611b0d8284611a56565b915081905092915050565b5f819050602082019050919050565b5f7fffffffff0000000000000000000000000000000000000000000000000000000082169050919050565b5f611b5d8251611b27565b80915050919050565b5f82821b905092915050565b5f611b7c82611a3e565b82611b8684611b18565b9050611b9181611b52565b92506004821015611bd157611bcc7fffffffff0000000000000000000000000000000000000000000000000000000083600403600802611b66565b831692505b505091905056fea26469706673582212204eb4841e25d14bc4e26038a7b1b3a1779499a375b976ea6023796dfa54c728b364736f6c634300081a0033")]
    contract ERC20 {

        mapping(address => uint256) public balanceOf;
        mapping(address => mapping(address => uint256)) public allowance;

        event Transfer(address indexed from, address indexed to, uint256 value);
        event Approval(address indexed owner, address indexed spender, uint256 value);

        using EVM for address;
        using EVM for address payable;

        // function ChainAddress(uint256 chainId, ERC20 contractAddr) internal view returns (ERC20) {
        //   return ERC20(address(contractAddr).onChain(chainId));
        // }

        // function on(uint256 chainId) internal view returns (ERC20) {
        //     return ChainAddress(chainId, this);
        // }

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

        function sendETH(uint256 chain, address payable to) public payable returns (bool) {
            (bool success, ) = to.onChain(chain).call{value: msg.value}("");
            require(success);
            return succcess;
        }

        function sendETHFrom(uint256 fromChain, uint256 toChain, address payable to) external payable returns (bool) {
            return on(fromChain).sendETH{value: msg.value}(toChain, to);
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
        alice.on_chain(A),
        AccountInfo::new(U256::from(10000), 0, KECCAK_EMPTY, Bytecode::default()),
    );
    db.insert_account_info(
        alice.on_chain(B),
        AccountInfo::new(U256::from(10000), 0, KECCAK_EMPTY, Bytecode::default()),
    );
    db.insert_account_info(
        bob.on_chain(A),
        AccountInfo::new(U256::from(10000), 0, KECCAK_EMPTY, Bytecode::default()),
    );
    db.insert_account_info(
        operator.on_chain(B),
        AccountInfo::new(U256::from(10000), 0, KECCAK_EMPTY, Bytecode::default()),
    );
    db.insert_account_info(
        operator.on_chain(A),
        AccountInfo::new(U256::from(10000), 0, KECCAK_EMPTY, Bytecode::default()),
    );
    deploy_contract(
        &mut db,
        operator.on_chain(A),
        deployment.on_chain(A),
        ERC20::BYTECODE.clone(),
    );
    // deploy_contract(
    //     &mut db,
    //     operator.on_chain(B),
    //     deployment.on_chain(B),
    //     ERC20::BYTECODE.clone(),
    // );

    let mut do_transact = |addr: ChainAddress, op: Vec<u8>, value: U256| -> ExecutionResult {
        println!("\n\n");
        let mut evm = Evm::builder()
            .modify_cfg_env(|c| {
                c.xchain = true;
                c.parent_chain_id = Some(A);
            })
            .modify_tx_env(|tx| {
                tx.caller = addr;
                tx.transact_to = TransactTo::Call(ChainAddress(addr.0, deployment));
                tx.data = op.clone().into();
                tx.value = value;
                tx.gas_limit = 30_000_000;
                tx.gas_price = U256::ZERO;
                tx.chain_ids = Some(vec![A, B]);
            })
            .with_db(&mut db)
            .build();
        let res = evm.transact().unwrap().result;
        evm.transact_commit().unwrap();
        drop(evm);
        //println!("{:?}\n~~~~~~~~~\n", res);

        if let ExecutionResult::Success { reason, gas_used, gas_refunded, logs, output, state_changes, gas_used_per_chain, gas_refunded_per_chain } = res.clone() {
            // for entry in state_changes.entries.iter() {
            //     println!("- {:?}", entry);
            // }
            // println!("*****************");
            // let state_diff = create_state_diff(state_changes, 1);
            // for entry in state_diff.entries.iter() {
            //     println!("- {:?}", entry);
            // }
            // println!("out");
            // for output in state_diff.outputs.iter() {
            //     println!("- {:?}", output);
            // }
            println!("gas used: {:?}", gas_used);
            println!("gas used per chain: {:?}", gas_used_per_chain);
        }

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
            U256::ZERO,
        ),
        do_transact(
            operator.on_chain(A),
            ERC20::xTransferCall {
                chain: Uint::from(B),
                to: operator,
                value: Uint::from(19999),
            }
            .abi_encode(),
            U256::ZERO,
        ),
        // do_transact(
        //     operator.on_chain(B),
        //     ERC20::transferCall {
        //         to: alice,
        //         value: Uint::from(9876),
        //     }
        //     .abi_encode(),
        //     U256::ZERO,
        // ),
        // do_transact(
        //     operator.on_chain(A),
        //     ERC20::transferCall {
        //         to: bob,
        //         value: Uint::from(9876),
        //     }
        //     .abi_encode(),
        //     U256::ZERO,
        // ),
        // // Give Bob some money
        // do_transact(
        //     operator.on_chain(B),
        //     ERC20::transferCall {
        //         to: bob,
        //         value: Uint::from(1234),
        //     }
        //     .abi_encode(),
        //     U256::ZERO,
        // ),
        // From A, Alice xTransfer 666 to Bob on B.
        do_transact(
            alice.on_chain(A),
            ERC20::xTransferCall {
                chain: Uint::from(B),
                to: bob,
                value: Uint::from(666),
            }
            .abi_encode(),
            U256::ZERO,
        ),
        // // From B, Bob xApprove Alice to spend his 666 on A.
        // do_transact(
        //     bob.on_chain(B),
        //     ERC20::xApproveCall {
        //         chain: Uint::from(A),
        //         spender: alice,
        //         value: Uint::from(666),
        //     }
        //     .abi_encode(),
        //     U256::ZERO,
        // ),
        // // From B, Bob checks if he transfer 555 to Alice on A in sandboxed mode.
        // do_transact(
        //     bob.on_chain(B),
        //     ERC20::sandboxedTransferCall {
        //         chain: Uint::from(A),
        //         to: alice,
        //         value: Uint::from(666),
        //     }
        //     .abi_encode(),
        //     U256::ZERO,
        // ),
        // // From A, Alice transfer Bob's 666 to herself.
        // do_transact(
        //     alice.on_chain(A),
        //     ERC20::transferFromCall {
        //         from: bob,
        //         to: alice,
        //         value: Uint::from(666),
        //     }
        //     .abi_encode(),
        //     U256::ZERO,
        // ),
        // L1 -> L2
        do_transact(
            alice.on_chain(A),
            ERC20::sendETHCall {
                chain: Uint::from(B),
                to: bob,
            }
            .abi_encode(),
            U256::from(123),
        ),
        // L2 -> L1
        do_transact(
            alice.on_chain(B),
            ERC20::sendETHCall {
                chain: Uint::from(A),
                to: bob,
            }
            .abi_encode(),
            U256::from(123),
        ),
        // L2 -> L1 -> L2
        do_transact(
            alice.on_chain(B),
            ERC20::sendETHFromCall {
                fromChain: Uint::from(A),
                toChain: Uint::from(B),
                to: bob,
            }
            .abi_encode(),
            U256::from(456),
        ),
    ];

    let balance = ERC20::balanceOfCall {
        _0: alice,
    };

   let balance = db.load_account(alice.on_chain(B)).unwrap().info.balance.as_limbs()[0];

    for res in results {
        //println!("res: {:?}", res);
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
        .modify_cfg_env(|cfg: &mut revm::primitives::CfgEnv| {
            cfg.parent_chain_id = Some(A);
        })
        .modify_tx_env(|tx| {
            tx.caller = deployer;
            tx.transact_to = TransactTo::Create;
            tx.data = code;
        })
        .with_db(cache_db)
        .build();
    let res = evm.transact().unwrap();
    //println!("res: {:?}", res);
    assert!(res.result.is_success());
    evm.transact_commit().unwrap();
    drop(evm);
}
