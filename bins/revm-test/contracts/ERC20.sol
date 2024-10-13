// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.8.2 <0.9.0;

import "./EVM.sol";


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


    function transferFrom(address from, address to, uint256 value) public returns (bool) {  
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