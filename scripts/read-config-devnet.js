const anchor = require('@coral-xyz/anchor');
const fs = require('fs');
const path = require('path');

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const idlPath = path.join(__dirname, '..', 'target', 'idl', 'project_comments.json');
  const idl = JSON.parse(fs.readFileSync(idlPath, 'utf8'));
  idl.address = '3C9uAwPX6Bbx2CybpPa1pWA7kFh5xsQLCPA4hbvkHDWE';
  const program = new anchor.Program(idl, provider);

  const [configPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from('config')],
    program.programId,
  );

  const config = await program.account.config.fetch(configPda);
  console.log('config PDA:', configPda.toBase58());
  console.log(JSON.stringify(config, null, 2));
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
