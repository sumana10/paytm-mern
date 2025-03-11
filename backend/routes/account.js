// backend/routes/account.js
const express = require('express');
const { authMiddleware } = require('../middleware');
const { Account } = require('../db');
const { default: mongoose } = require('mongoose');

const router = express.Router();

router.get("/balance", authMiddleware, async (req, res) => {
    const account = await Account.findOne({
        userId: req.userId
    });

    res.json({
        balance: account.balance
    })
});

// router.post("/transfer", authMiddleware, async (req, res) => {
//     const session = await mongoose.startSession();

//     session.startTransaction();
//     const { amount, to } = req.body;

//     // Fetch the accounts within the transaction
//     const account = await Account.findOne({ userId: req.userId }).session(session);

//     if (!account || account.balance < amount) {
//         await session.abortTransaction();
//         return res.status(400).json({
//             message: "Insufficient balance"
//         });
//     }

//     const toAccount = await Account.findOne({ userId: to }).session(session);

//     if (!toAccount) {
//         await session.abortTransaction();
//         return res.status(400).json({
//             message: "Invalid account"
//         });
//     }

//     // Perform the transfer
//     await Account.updateOne({ userId: req.userId }, { $inc: { balance: -amount } }).session(session);
//     await Account.updateOne({ userId: to }, { $inc: { balance: amount } }).session(session);

//     // Commit the transaction
//     await session.commitTransaction();
//     res.json({
//         message: "Transfer successful"
//     });
// });

router.post("/transfer", authMiddleware, async (req, res) => {
    const session = await mongoose.startSession();
    session.startTransaction();

    try {
        const { amount, to } = req.body;

        // Step 1: Deduct balance using findOneAndUpdate (atomic)
        const account = await Account.findOneAndUpdate(
            { userId: req.userId, balance: { $gte: amount } }, // Ensure sufficient balance
            { $inc: { balance: -amount } },
            { session, new: true }
        );

        if (!account) {
            throw new Error("Insufficient balance");
        }

        // Step 2: Credit balance to recipient (atomic)
        const toAccount = await Account.findOneAndUpdate(
            { userId: to },
            { $inc: { balance: amount } },
            { session, new: true }
        );

        if (!toAccount) {
            throw new Error("Invalid account");
        }

        // Step 3: Commit transaction
        await session.commitTransaction();
        res.json({ message: "Transfer successful" });
    } catch (error) {
        await session.abortTransaction();
        res.status(400).json({ message: error.message });
    } finally {
        session.endSession();
    }
});


module.exports = router;