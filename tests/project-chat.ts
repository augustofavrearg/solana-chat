import * as anchor from "@coral-xyz/anchor";

describe("project-chat", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  it("bootstraps test harness", async () => {
    // Contract behavior is covered primarily by Rust unit tests.
  });
});
