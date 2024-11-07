// SPDX-License-Identifier: MIT

pragma solidity >=0.8.12 <0.9.0;

import "./EVM.sol";

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