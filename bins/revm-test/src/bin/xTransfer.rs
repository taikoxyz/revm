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
    #[sol(bytecode="0x6080604052348015600e575f80fd5b506201869f5f803373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20819055506119e5806100605f395ff3fe6080604052600436106100c1575f3560e01c806370a082311161007e578063c664c71411610058578063c664c714146102e1578063cb05d551146102fd578063d48024a714610319578063dd62ed3e14610355576100c1565b806370a082311461022d578063a9059cbb14610269578063b3f322af146102a5576100c1565b8063095ea7b3146100c5578063104e81ff1461010157806323b872dd1461013d578063297926631461017957806330e0789e146101b55780634e6ec247146101f1575b5f80fd5b3480156100d0575f80fd5b506100eb60048036038101906100e691906111cd565b610391565b6040516100f8919061121a565b60405180910390f35b34801561010c575f80fd5b5061012760048036038101906101229190611233565b61047d565b604051610134919061121a565b60405180910390f35b348015610148575f80fd5b50610163600480360381019061015e9190611233565b6105d8565b604051610170919061121a565b60405180910390f35b348015610184575f80fd5b5061019f600480360381019061019a9190611283565b61091c565b6040516101ac919061121a565b60405180910390f35b3480156101c0575f80fd5b506101db60048036038101906101d69190611233565b6109ad565b6040516101e8919061121a565b60405180910390f35b3480156101fc575f80fd5b50610217600480360381019061021291906111cd565b610aca565b604051610224919061121a565b60405180910390f35b348015610238575f80fd5b50610253600480360381019061024e91906112d3565b610b94565b604051610260919061121a565b60405180910390f35b348015610274575f80fd5b5061028f600480360381019061028a91906111cd565b610ba8565b60405161029c919061121a565b60405180910390f35b3480156102b0575f80fd5b506102cb60048036038101906102c69190611283565b610d3a565b6040516102d8919061121a565b60405180910390f35b6102fb60048036038101906102f69190611339565b610dcd565b005b61031760048036038101906103129190611377565b610e6b565b005b348015610324575f80fd5b5061033f600480360381019061033a9190611283565b610ee2565b60405161034c919061121a565b60405180910390f35b348015610360575f80fd5b5061037b600480360381019061037691906113c7565b610fc3565b604051610388919061121a565b60405180910390f35b5f8160015f3373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20819055508273ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b9258460405161046c919061121a565b60405180910390a381905092915050565b5f3073ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff16146104ec576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016104e390611485565b60405180910390fd5b8160015f8673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20819055508273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925846040516105c6919061121a565b60405180910390a38190509392505050565b5f815f808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20541015610658576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161064f906114ed565b60405180910390fd5b3373ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff1614610747578160015f8673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f3373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20541015610746576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161073d90611555565b60405180910390fd5b5b815f808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f82825461079291906115a0565b92505081905550815f808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8282546107e491906115d3565b925050819055503373ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff16146108ad578160015f8673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f3373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8282546108a591906115a0565b925050819055505b8273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef8460405161090a919061121a565b60405180910390a38190509392505050565b5f61092684610fe3565b503073ffffffffffffffffffffffffffffffffffffffff1663104e81ff3385856040518463ffffffff1660e01b815260040161096493929190611615565b6020604051808303815f875af1158015610980573d5f803e3d5ffd5b505050506040513d601f19601f820116820180604052508101906109a4919061165e565b90509392505050565b5f3073ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff1614610a1c576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610a13906116d3565b60405180910390fd5b815f808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f828254610a6791906115a0565b92505081905550815f808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f828254610ab991906115d3565b925050819055508190509392505050565b5f3073ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff1614610b39576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610b30906116d3565b60405180910390fd5b815f808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f828254610b8491906115d3565b9250508190555081905092915050565b5f602052805f5260405f205f915090505481565b5f815f803373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20541015610c28576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610c1f906114ed565b60405180910390fd5b815f803373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f828254610c7391906115a0565b92505081905550815f808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f828254610cc591906115d3565b925050819055508273ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef84604051610d29919061121a565b60405180910390a381905092915050565b5f610d46846001610ff5565b503073ffffffffffffffffffffffffffffffffffffffff166330e0789e3385856040518463ffffffff1660e01b8152600401610d8493929190611615565b6020604051808303815f875af1158015610da0573d5f803e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610dc4919061165e565b90509392505050565b5f610df7838373ffffffffffffffffffffffffffffffffffffffff1661100a90919063ffffffff16565b73ffffffffffffffffffffffffffffffffffffffff1634604051610e1a9061171e565b5f6040518083038185875af1925050503d805f8114610e54576040519150601f19603f3d011682016040523d82523d5f602084013e610e59565b606091505b5050905080610e66575f80fd5b505050565b610e7483611022565b73ffffffffffffffffffffffffffffffffffffffff1663c664c7143484846040518463ffffffff1660e01b8152600401610eaf929190611741565b5f604051808303818588803b158015610ec6575f80fd5b505af1158015610ed8573d5f803e3d5ffd5b5050505050505050565b5f815f803373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f828254610f2e91906115a0565b92505081905550610f3e84610fe3565b503073ffffffffffffffffffffffffffffffffffffffff16634e6ec24784846040518363ffffffff1660e01b8152600401610f7a929190611768565b6020604051808303815f875af1158015610f96573d5f803e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610fba919061165e565b90509392505050565b6001602052815f5260405f20602052805f5260405f205f91509150505481565b5f610fee825f610ff5565b9050919050565b5f61100283833230611034565b905092915050565b5f80611016835f610ff5565b90508391505092915050565b5f61102d823061105d565b9050919050565b5f611053858585855f801b60405180602001604052805f81525061108f565b9050949350505050565b5f611087838373ffffffffffffffffffffffffffffffffffffffff1661100a90919063ffffffff16565b905092915050565b5f8060018888888888886040516020016110af979695949392919061191d565b60405160208183030381529060405290505f806104d273ffffffffffffffffffffffffffffffffffffffff16836040516110e99190611999565b5f60405180830381855afa9150503d805f8114611121576040519150601f19603f3d011682016040523d82523d5f602084013e611126565b606091505b5091509150600193505050509695505050505050565b5f80fd5b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f61116982611140565b9050919050565b6111798161115f565b8114611183575f80fd5b50565b5f8135905061119481611170565b92915050565b5f819050919050565b6111ac8161119a565b81146111b6575f80fd5b50565b5f813590506111c7816111a3565b92915050565b5f80604083850312156111e3576111e261113c565b5b5f6111f085828601611186565b9250506020611201858286016111b9565b9150509250929050565b6112148161119a565b82525050565b5f60208201905061122d5f83018461120b565b92915050565b5f805f6060848603121561124a5761124961113c565b5b5f61125786828701611186565b935050602061126886828701611186565b9250506040611279868287016111b9565b9150509250925092565b5f805f6060848603121561129a5761129961113c565b5b5f6112a7868287016111b9565b93505060206112b886828701611186565b92505060406112c9868287016111b9565b9150509250925092565b5f602082840312156112e8576112e761113c565b5b5f6112f584828501611186565b91505092915050565b5f61130882611140565b9050919050565b611318816112fe565b8114611322575f80fd5b50565b5f813590506113338161130f565b92915050565b5f806040838503121561134f5761134e61113c565b5b5f61135c858286016111b9565b925050602061136d85828601611325565b9150509250929050565b5f805f6060848603121561138e5761138d61113c565b5b5f61139b868287016111b9565b93505060206113ac868287016111b9565b92505060406113bd86828701611325565b9150509250925092565b5f80604083850312156113dd576113dc61113c565b5b5f6113ea85828601611186565b92505060206113fb85828601611186565b9150509250929050565b5f82825260208201905092915050565b7f4f6e6c7920636f6e747261637420697473656c662063616e2063616c6c2074685f8201527f69732066756e6374696f6e000000000000000000000000000000000000000000602082015250565b5f61146f602b83611405565b915061147a82611415565b604082019050919050565b5f6020820190508181035f83015261149c81611463565b9050919050565b7f496e73756666696369656e742062616c616e63650000000000000000000000005f82015250565b5f6114d7601483611405565b91506114e2826114a3565b602082019050919050565b5f6020820190508181035f830152611504816114cb565b9050919050565b7f416c6c6f77616e636520657863656564656400000000000000000000000000005f82015250565b5f61153f601283611405565b915061154a8261150b565b602082019050919050565b5f6020820190508181035f83015261156c81611533565b9050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b5f6115aa8261119a565b91506115b58361119a565b92508282039050818111156115cd576115cc611573565b5b92915050565b5f6115dd8261119a565b91506115e88361119a565b9250828201905080821115611600576115ff611573565b5b92915050565b61160f8161115f565b82525050565b5f6060820190506116285f830186611606565b6116356020830185611606565b611642604083018461120b565b949350505050565b5f81519050611658816111a3565b92915050565b5f602082840312156116735761167261113c565b5b5f6116808482850161164a565b91505092915050565b7f4f6e6c79207468697320636f6e74726163742063616e206d696e7400000000005f82015250565b5f6116bd601b83611405565b91506116c882611689565b602082019050919050565b5f6020820190508181035f8301526116ea816116b1565b9050919050565b5f81905092915050565b50565b5f6117095f836116f1565b9150611714826116fb565b5f82019050919050565b5f611728826116fe565b9150819050919050565b61173b816112fe565b82525050565b5f6040820190506117545f83018561120b565b6117616020830184611732565b9392505050565b5f60408201905061177b5f830185611606565b611788602083018461120b565b9392505050565b5f61ffff82169050919050565b5f8160f01b9050919050565b5f6117b28261179c565b9050919050565b6117ca6117c58261178f565b6117a8565b82525050565b5f67ffffffffffffffff82169050919050565b5f8160c01b9050919050565b5f6117f9826117e3565b9050919050565b61181161180c826117d0565b6117ef565b82525050565b5f8115159050919050565b5f8160f81b9050919050565b5f61183882611822565b9050919050565b5f6118498261182e565b9050919050565b61186161185c82611817565b61183f565b82525050565b5f8160601b9050919050565b5f61187d82611867565b9050919050565b5f61188e82611873565b9050919050565b6118a66118a18261115f565b611884565b82525050565b5f819050919050565b5f819050919050565b6118cf6118ca826118ac565b6118b5565b82525050565b5f81519050919050565b8281835e5f83830152505050565b5f6118f7826118d5565b61190181856116f1565b93506119118185602086016118df565b80840191505092915050565b5f611928828a6117b9565b6002820191506119388289611800565b6008820191506119488288611850565b6001820191506119588287611895565b6014820191506119688286611895565b60148201915061197882856118be565b60208201915061198882846118ed565b915081905098975050505050505050565b5f6119a482846118ed565b91508190509291505056fea2646970667358221220b7b8588ba321af14e219941dd35cdbf1281b97527f0b1c0ce78e781c3e180dd464736f6c634300081a0033")]
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

        function sendETH(uint256 chain, address payable to) public payable {
            (bool success, ) = to.onChain(chain).call{value: msg.value}("");
            require(success);
        }

        function sendETHFrom(uint256 fromChain, uint256 toChain, address payable to) external payable {
            on(fromChain).sendETH{value: msg.value}(toChain, to);
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
    deploy_contract(
        &mut db,
        operator.on_chain(B),
        deployment.on_chain(B),
        ERC20::BYTECODE.clone(),
    );

    let mut do_transact = |addr: ChainAddress, op: Vec<u8>, value: U256| -> ExecutionResult {
        println!("\n\n");
        let mut evm = Evm::builder()
            .modify_cfg_env(|c| {
                c.xchain = true;
            })
            .modify_tx_env(|tx| {
                tx.caller = addr;
                tx.transact_to = TransactTo::Call(ChainAddress(addr.0, deployment));
                tx.data = op.clone().into();
                tx.value = value;
                tx.gas_price = U256::ZERO;
            })
            .with_db(&mut db)
            .build();
        let res = evm.transact().unwrap().result;
        evm.transact_commit().unwrap();
        drop(evm);
        println!("{:?}\n~~~~~~~~~\n", res);

        // if let ExecutionResult::Success { reason, gas_used, gas_refunded, logs, output, state_changes } = res.clone() {
        //     for entry in state_changes.entries.iter() {
        //         println!("- {:?}", entry);
        //     }
        //     println!("*****************");
        //     let state_diff = create_state_diff(state_changes, 1);
        //     for entry in state_diff.entries.iter() {
        //         println!("- {:?}", entry);
        //     }
        // }

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
            operator.on_chain(B),
            ERC20::transferCall {
                to: alice,
                value: Uint::from(9876),
            }
            .abi_encode(),
            U256::ZERO,
        ),
        do_transact(
            operator.on_chain(A),
            ERC20::transferCall {
                to: bob,
                value: Uint::from(9876),
            }
            .abi_encode(),
            U256::ZERO,
        ),
        // Give Bob some money
        do_transact(
            operator.on_chain(B),
            ERC20::transferCall {
                to: bob,
                value: Uint::from(1234),
            }
            .abi_encode(),
            U256::ZERO,
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
            U256::ZERO,
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
            U256::ZERO,
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
            U256::ZERO,
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
            U256::ZERO,
        ),
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
        // L2 -> L1 -> L1
        do_transact(
            alice.on_chain(B),
            ERC20::sendETHFromCall {
                fromChain: Uint::from(A),
                toChain: Uint::from(A),
                to: bob,
            }
            .abi_encode(),
            U256::from(123),
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
    let res = evm.transact().unwrap();
    //println!("res: {:?}", res);
    assert!(res.result.is_success());
    evm.transact_commit().unwrap();
    drop(evm);
}
