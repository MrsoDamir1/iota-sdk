/**
 * This example backups your data in a secure file.
 * You can move this file to another app or device and restore it.
 */
const path = require('path')
require('dotenv').config({ path: path.resolve(__dirname, '.env') });
const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();
        await manager.backup('./backup', process.env.STRONGHOLD_PASSWORD);
        console.log('Successfully created backup');
    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
