const anchor = require('@coral-xyz/anchor');
const fs = require('fs');
const path = require('path');

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const idlPath = path.join(__dirname, '..', 'target', 'idl', 'project_chat.json');
  const idl = JSON.parse(fs.readFileSync(idlPath, 'utf8'));
  idl.address = '3C9uAwPX6Bbx2CybpPa1pWA7kFh5xsQLCPA4hbvkHDWE';
  const program = new anchor.Program(idl, provider);

  const [configPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from('config')],
    program.programId,
  );

  const existing = await provider.connection.getAccountInfo(configPda);
  if (existing) {
    console.log(`Config already initialized: ${configPda.toBase58()}`);
    return;
  }

  const walletPk = provider.wallet.publicKey;
  const txSig = await program.methods
    .initializeConfig(walletPk, 0, 500, 50)
    .accounts({
      config: configPda,
      payer: walletPk,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc({ commitment: 'confirmed' });

  console.log('initialize_config tx:', txSig);
  console.log('config PDA:', configPda.toBase58());
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
