import './style.css'

interface Exploit {
  id: string
  name: string
  protocol: string
  date: string
  loss: string
  lossAmount: number
  description: string
  tags: string[]
  attackSteps: { title: string; desc: string }[]
  vulnerableCode: string
  secureCode: string
  keyInsight: string
}

const exploits: Exploit[] = [
  {
    id: 'infinite-mint',
    name: 'Infinite Mint',
    protocol: 'Cashio (March 2022)',
    date: 'March 2022',
    loss: '$52M',
    lossAmount: 52,
    description:
      'Stablecoin minting accepted any SPL token as collateral without validation, allowing attackers to mint unlimited CASH tokens.',
    tags: ['Stablecoin', 'Collateral Bypass', 'Token Validation'],
    attackSteps: [
      {
        title: 'Create worthless token',
        desc: 'Attacker deploys a new SPL token with unlimited supply',
      },
      {
        title: 'Deposit fake collateral',
        desc: 'Deposit the worthless token into the lending protocol',
      },
      {
        title: 'Mint stablecoins',
        desc: 'Protocol accepts deposit and mints 1:1 stablecoins',
      },
      {
        title: 'Dump stablecoins',
        desc: 'Sell minted stablecoins on DEXs for real value',
      },
    ],
    vulnerableCode: `<span class="keyword">pub fn</span> <span class="function">mint_insecure</span>(ctx: Context&lt;MintInsecure&gt;, amount: u64) -> Result&lt;()&gt; {
    <span class="comment">// BUG: We check that collateral was deposited,</span>
    <span class="comment">// but NOT what token it is!</span>
    <span class="bug">// Any SPL token is accepted as valid collateral.</span>
    
    <span class="keyword">require!</span>(
        ctx.accounts.user_collateral.amount >= amount,
        ErrorCode::InsufficientCollateral
    );

    <span class="comment">// Mint stablecoins 1:1 against "collateral"</span>
    token::mint_to(..., amount)?;

    <span class="function">msg!</span>(<span class="string">"Minted stablecoins (no validation)"</span>);
    <span class="keyword">Ok</span>(())
}`,
    secureCode: `<span class="keyword">pub fn</span> <span class="function">mint_secure</span>(ctx: Context&lt;MintSecure&gt;, amount: u64) -> Result&lt;()&gt; {
    <span class="comment">// FIX: Validate the collateral mint!</span>
    <span class="fix">let collateral_mint = ctx.accounts.user_collateral.mint;</span>
    <span class="fix">require!(</span>
    <span class="fix">    collateral_mint == ctx.accounts.config.approved_collateral,</span>
    <span class="fix">    ErrorCode::InvalidCollateral</span>
    <span class="fix">);</span>
    
    <span class="keyword">require!</span>(
        ctx.accounts.user_collateral.amount >= amount,
        ErrorCode::InsufficientCollateral
    );

    token::mint_to(..., amount)?;
    <span class="function">msg!</span>(<span class="string">"Minted with validated collateral"</span>);
    <span class="keyword">Ok</span>(())
}`,
    keyInsight:
      "Always validate token mints. Use Anchor's token::mint constraint or manual checks to ensure only approved tokens are accepted as collateral.",
  },
  {
    id: 'oracle-manipulation',
    name: 'Oracle Manipulation',
    protocol: 'Mango Markets (October 2022)',
    date: 'October 2022',
    loss: '$116M',
    lossAmount: 116,
    description:
      'Lending protocol trusted oracle prices without staleness or confidence checks, enabling price manipulation attacks.',
    tags: ['DeFi Lending', 'Oracle', 'Price Manipulation'],
    attackSteps: [
      {
        title: 'Deposit initial collateral',
        desc: 'Attacker deposits $5M USDC as collateral',
      },
      {
        title: 'Open large perpetual position',
        desc: 'Take a massive long position on MNGO token',
      },
      {
        title: 'Pump spot price',
        desc: 'Buy MNGO across multiple exchanges, pumping price 2,394%',
      },
      {
        title: 'Borrow against inflated collateral',
        desc: 'Oracle reflects pumped price, collateral value skyrockets',
      },
      {
        title: 'Withdraw borrowed funds',
        desc: 'Borrow $116M and withdraw before price crashes',
      },
    ],
    vulnerableCode: `<span class="keyword">pub fn</span> <span class="function">borrow_insecure</span>(ctx: Context&lt;BorrowInsecure&gt;, amount: u64) -> Result&lt;()&gt; {
    <span class="keyword">let</span> oracle = &ctx.accounts.oracle;
    
    <span class="comment">// BUG: Blindly trust oracle price without ANY validation</span>
    <span class="bug">let collateral_value = user.deposited * oracle.price;</span>
    
    <span class="keyword">let</span> max_borrow = collateral_value * 80 / 100; <span class="comment">// 80% LTV</span>
    
    <span class="keyword">require!</span>(amount <= max_borrow, ErrorCode::Insufficient);

    ctx.accounts.user_position.borrowed += amount;
    <span class="keyword">Ok</span>(())
}`,
    secureCode: `<span class="keyword">pub fn</span> <span class="function">borrow_secure</span>(ctx: Context&lt;BorrowSecure&gt;, amount: u64) -> Result&lt;()&gt; {
    <span class="keyword">let</span> oracle = &ctx.accounts.oracle;
    <span class="keyword">let</span> current_slot = Clock::get()?.slot;

    <span class="fix">// FIX 1: Reject stale prices (>10 slots)</span>
    <span class="fix">require!(</span>
    <span class="fix">    current_slot - oracle.last_update_slot <= 10,</span>
    <span class="fix">    ErrorCode::StaleOracle</span>
    <span class="fix">);</span>

    <span class="fix">// FIX 2: Reject low confidence data</span>
    <span class="fix">require!(</span>
    <span class="fix">    oracle.confidence * 100 / oracle.price <= 5,</span>
    <span class="fix">    ErrorCode::LowConfidence</span>
    <span class="fix">);</span>

    <span class="fix">// FIX 3: Circuit breaker for extreme moves</span>
    <span class="fix">let price_change = calculate_change(oracle.price, last_price);</span>
    <span class="fix">require!(price_change <= 50, ErrorCode::ExtremePrice);</span>

    <span class="keyword">let</span> collateral_value = user.deposited * oracle.price;
    ...
}`,
    keyInsight:
      'Never trust oracle prices blindly. Implement staleness checks, confidence interval validation, and circuit breakers for extreme price movements.',
  },
  {
    id: 'sysvar-spoofing',
    name: 'Sysvar Spoofing',
    protocol: 'Wormhole (February 2022)',
    date: 'February 2022',
    loss: '$325M',
    lossAmount: 325,
    description:
      "Bridge used deprecated signature verification that didn't validate the Instructions sysvar address, allowing attackers to forge signatures.",
    tags: ['Bridge', 'Signature Verification', 'Sysvar'],
    attackSteps: [
      {
        title: 'Create fake account',
        desc: 'Deploy account with crafted data mimicking sysvar structure',
      },
      {
        title: 'Craft fake signatures',
        desc: 'Write data that makes verification logic return "valid"',
      },
      {
        title: 'Call verify function',
        desc: 'Pass fake account as Instructions sysvar',
      },
      {
        title: 'Mint bridged tokens',
        desc: 'Verification passes, attacker mints 120k ETH',
      },
    ],
    vulnerableCode: `<span class="keyword">pub fn</span> <span class="function">verify_and_mint_insecure</span>(
    ctx: Context&lt;VerifyInsecure&gt;,
    amount: u64,
) -> Result&lt;()&gt; {
    <span class="comment">// BUG: Accept ANY account as instructions sysvar</span>
    <span class="bug">let instructions_account = &ctx.accounts.instructions;</span>
    
    <span class="comment">// Attacker can make this return anything</span>
    <span class="keyword">let</span> is_verified = verify_signatures(instructions_account)?;
    
    <span class="keyword">require!</span>(is_verified, ErrorCode::InvalidSignature);

    <span class="comment">// Mint tokens to user</span>
    ctx.accounts.bridge.total_minted += amount;
    <span class="keyword">Ok</span>(())
}

<span class="keyword">#[derive(Accounts)]</span>
<span class="keyword">pub struct</span> <span class="type">VerifyInsecure</span>&lt;'info&gt; {
    <span class="bug">/// CHECK: No address verification!</span>
    <span class="bug">pub instructions: UncheckedAccount&lt;'info&gt;,</span>
    ...
}`,
    secureCode: `<span class="keyword">pub fn</span> <span class="function">verify_and_mint_secure</span>(
    ctx: Context&lt;VerifySecure&gt;,
    amount: u64,
) -> Result&lt;()&gt; {
    <span class="comment">// FIX: Address is constrained in accounts struct</span>
    <span class="keyword">let</span> is_verified = verify_signatures(&ctx.accounts.instructions)?;
    
    <span class="keyword">require!</span>(is_verified, ErrorCode::InvalidSignature);
    ctx.accounts.bridge.total_minted += amount;
    <span class="keyword">Ok</span>(())
}

<span class="keyword">#[derive(Accounts)]</span>
<span class="keyword">pub struct</span> <span class="type">VerifySecure</span>&lt;'info&gt; {
    <span class="fix">/// FIX: Constrain to actual sysvar address</span>
    <span class="fix">#[account(address = sysvar::instructions::ID)]</span>
    <span class="fix">pub instructions: UncheckedAccount&lt;'info&gt;,</span>
    ...
}`,
    keyInsight:
      "Always use address constraints for sysvars. Prefer Anchor's Sysvar<'info, Instructions> wrapper which enforces the correct address automatically.",
  },
  {
    id: 'reentrancy-cpi',
    name: 'Reentrancy via CPI',
    protocol: 'Crema Finance (July 2022)',
    date: 'July 2022',
    loss: '$9M',
    lossAmount: 9,
    description:
      'DEX updated pool state AFTER making external CPI calls, allowing attackers to re-enter with stale state.',
    tags: ['DEX', 'AMM', 'Reentrancy', 'CPI'],
    attackSteps: [
      {
        title: 'Call swap function',
        desc: 'Initiate a swap on the vulnerable DEX',
      },
      {
        title: 'Receive CPI callback',
        desc: 'DEX makes external call before updating state',
      },
      {
        title: 'Re-enter swap',
        desc: "Attacker's contract calls swap again with old reserves",
      },
      {
        title: 'Extract excess tokens',
        desc: 'Multiple swaps execute against stale state',
      },
    ],
    vulnerableCode: `<span class="keyword">pub fn</span> <span class="function">swap_insecure</span>(ctx: Context&lt;SwapInsecure&gt;, amount_in: u64) -> Result&lt;()&gt; {
    <span class="keyword">let</span> pool = &ctx.accounts.pool;
    
    <span class="comment">// Calculate output based on current reserves</span>
    <span class="keyword">let</span> amount_out = calculate_output(amount_in, pool.reserve_a, pool.reserve_b);
    
    <span class="bug">// BUG: External call BEFORE updating state!</span>
    <span class="bug">// Attacker can re-enter with old reserves</span>
    <span class="bug">simulate_external_cpi(&ctx.accounts.callback_program)?;</span>

    <span class="comment">// State updated AFTER CPI - TOO LATE!</span>
    <span class="keyword">let</span> pool = &<span class="keyword">mut</span> ctx.accounts.pool;
    pool.reserve_a += amount_in;
    pool.reserve_b -= amount_out;
    <span class="keyword">Ok</span>(())
}`,
    secureCode: `<span class="keyword">pub fn</span> <span class="function">swap_secure</span>(ctx: Context&lt;SwapSecure&gt;, amount_in: u64) -> Result&lt;()&gt; {
    <span class="keyword">let</span> pool = &<span class="keyword">mut</span> ctx.accounts.pool;
    
    <span class="fix">// FIX 1: Reentrancy guard</span>
    <span class="fix">require!(!pool.is_locked, ErrorCode::Reentrancy);</span>
    <span class="fix">pool.is_locked = true;</span>
    
    <span class="keyword">let</span> amount_out = calculate_output(amount_in, pool.reserve_a, pool.reserve_b);
    
    <span class="fix">// FIX 2: Update state BEFORE external call</span>
    <span class="fix">pool.reserve_a += amount_in;</span>
    <span class="fix">pool.reserve_b -= amount_out;</span>
    
    <span class="comment">// Now safe to make external call</span>
    simulate_external_cpi(&ctx.accounts.callback_program)?;
    
    <span class="fix">pool.is_locked = false;</span>
    <span class="keyword">Ok</span>(())
}`,
    keyInsight:
      'Follow the checks-effects-interactions pattern: update state BEFORE making external CPI calls. Use reentrancy guards for critical functions.',
  },
  {
    id: 'access-control',
    name: 'Access Control',
    protocol: 'Raydium (December 2022)',
    date: 'December 2022',
    loss: '$4.4M',
    lossAmount: 4.4,
    description:
      'Admin functions allowed immediate execution with a single compromised key, enabling instant fund drainage.',
    tags: ['Admin', 'Key Management', 'Timelock'],
    attackSteps: [
      {
        title: 'Compromise admin key',
        desc: 'Attacker gains access to protocol admin private key',
      },
      {
        title: 'Call withdraw function',
        desc: "Single signature is all that's required",
      },
      {
        title: 'Drain treasury',
        desc: 'Funds transferred instantly with no delay',
      },
      { title: 'No time to react', desc: 'Team discovers loss after the fact' },
    ],
    vulnerableCode: `<span class="keyword">pub fn</span> <span class="function">withdraw_insecure</span>(ctx: Context&lt;WithdrawInsecure&gt;, amount: u64) -> Result&lt;()&gt; {
    <span class="comment">// Only check: is the signer the admin?</span>
    <span class="bug">// If admin key is compromised, attacker wins instantly</span>
    <span class="keyword">require!</span>(
        ctx.accounts.signer.key() == ctx.accounts.vault.admin,
        ErrorCode::Unauthorized
    );

    <span class="keyword">require!</span>(ctx.accounts.vault.balance >= amount, ErrorCode::Insufficient);

    <span class="bug">// IMMEDIATE withdrawal - no delay, no multisig</span>
    <span class="bug">ctx.accounts.vault.balance -= amount;</span>
    
    <span class="function">msg!</span>(<span class="string">"Withdrawn {} immediately"</span>, amount);
    <span class="keyword">Ok</span>(())
}`,
    secureCode: `<span class="keyword">pub fn</span> <span class="function">initiate_withdrawal</span>(ctx: ..., amount: u64, recipient: Pubkey) -> Result&lt;()&gt; {
    <span class="fix">// FIX: Two-phase withdrawal with timelock</span>
    <span class="fix">const TIMELOCK_SLOTS: u64 = 100; // ~40 seconds delay</span>
    
    <span class="keyword">let</span> vault = &<span class="keyword">mut</span> ctx.accounts.vault;
    <span class="fix">vault.pending_withdrawal = Some(PendingWithdrawal {</span>
    <span class="fix">    amount,</span>
    <span class="fix">    recipient,</span>
    <span class="fix">    executable_after: Clock::get()?.slot + TIMELOCK_SLOTS,</span>
    <span class="fix">});</span>
    
    <span class="function">msg!</span>(<span class="string">"Withdrawal initiated - can execute after timelock"</span>);
    <span class="keyword">Ok</span>(())
}

<span class="keyword">pub fn</span> <span class="function">cancel_withdrawal</span>(ctx: ...) -> Result&lt;()&gt; {
    <span class="fix">// Team can cancel if they detect compromise</span>
    <span class="fix">ctx.accounts.vault.pending_withdrawal = None;</span>
    <span class="keyword">Ok</span>(())
}`,
    keyInsight:
      'Implement timelocks for sensitive admin operations. This gives the team time to detect and respond to compromised keys before funds are lost.',
  },
]

function renderApp() {
  const totalLoss = exploits.reduce((sum, e) => sum + e.lossAmount, 0)

  const app = document.querySelector<HTMLDivElement>('#app')!
  app.innerHTML = `
    <section class="hero">
      <div class="container">
        <h1>Solana Security Lab</h1>
        <p class="subtitle">
          Learn from $500M+ in real-world exploits. Interactive code examples showing 
          vulnerable patterns and their secure alternatives.
        </p>
        <div class="stats">
          <div class="stat">
            <div class="value">${exploits.length}</div>
            <div class="label">Exploits Analyzed</div>
          </div>
          <div class="stat">
            <div class="value">$${totalLoss}M+</div>
            <div class="label">Total Losses</div>
          </div>
          <div class="stat">
            <div class="value">2022</div>
            <div class="label">Year of Incidents</div>
          </div>
        </div>
      </div>
    </section>

    <section class="exploits-section">
      <div class="container">
        <h2 class="section-title">Real-World Exploit Templates</h2>
        <div class="exploits-grid">
          ${exploits
            .map(
              (e) => `
            <div class="exploit-card" data-id="${e.id}">
              <div class="header">
                <span class="name">${e.name}</span>
                <span class="loss">${e.loss}</span>
              </div>
              <div class="protocol">${e.protocol}</div>
              <p class="description">${e.description}</p>
              <div class="tags">
                ${e.tags.map((t) => `<span class="tag">${t}</span>`).join('')}
              </div>
            </div>
          `,
            )
            .join('')}
        </div>
      </div>
    </section>

    <footer>
      <div class="container">
        Educational resource for Solana security research. 
        <a href="https://github.com/Don-Vicks/solana-program-templates" target="_blank">View on GitHub</a>
      </div>
    </footer>

    <div class="modal-overlay" id="modal-overlay">
      <div class="modal" id="modal">
        <div class="modal-header">
          <div class="title-row">
            <h2 id="modal-title"></h2>
            <button class="close-btn" id="close-modal">&times;</button>
          </div>
        </div>
        <div class="modal-body" id="modal-body"></div>
      </div>
    </div>
  `

  // Event listeners
  document.querySelectorAll('.exploit-card').forEach((card) => {
    card.addEventListener('click', () => {
      const id = card.getAttribute('data-id')
      const exploit = exploits.find((e) => e.id === id)
      if (exploit) openModal(exploit)
    })
  })

  document.getElementById('close-modal')?.addEventListener('click', closeModal)
  document.getElementById('modal-overlay')?.addEventListener('click', (e) => {
    if ((e.target as HTMLElement).id === 'modal-overlay') closeModal()
  })

  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') closeModal()
  })
}

function openModal(exploit: Exploit) {
  const overlay = document.getElementById('modal-overlay')
  const title = document.getElementById('modal-title')
  const body = document.getElementById('modal-body')

  if (!overlay || !title || !body) return

  title.textContent = exploit.name

  body.innerHTML = `
    <div class="info-grid">
      <div class="info-card">
        <div class="label">Protocol</div>
        <div class="value">${exploit.protocol.split('(')[0].trim()}</div>
      </div>
      <div class="info-card">
        <div class="label">Date</div>
        <div class="value">${exploit.date}</div>
      </div>
      <div class="info-card">
        <div class="label">Amount Lost</div>
        <div class="value red">${exploit.loss}</div>
      </div>
    </div>

    <div class="modal-section">
      <h3><span class="icon">⚔️</span> Attack Flow</h3>
      <div class="attack-flow">
        ${exploit.attackSteps
          .map(
            (step, i) => `
          <div class="attack-step">
            <div class="step-number">${i + 1}</div>
            <div class="step-content">
              <div class="title">${step.title}</div>
              <div class="desc">${step.desc}</div>
            </div>
          </div>
        `,
          )
          .join('')}
      </div>
    </div>

    <div class="modal-section">
      <h3><span class="icon">💀</span> Vulnerable Code</h3>
      <div class="code-block">
        <div class="code-header">
          <span class="filename">${exploit.id}.rs</span>
          <span class="status vulnerable">VULNERABLE</span>
        </div>
        <div class="code-content">
          <pre><code>${exploit.vulnerableCode}</code></pre>
        </div>
      </div>
    </div>

    <div class="modal-section">
      <h3><span class="icon">🛡️</span> Secure Implementation</h3>
      <div class="code-block">
        <div class="code-header">
          <span class="filename">${exploit.id}_secure.rs</span>
          <span class="status secure">SECURE</span>
        </div>
        <div class="code-content">
          <pre><code>${exploit.secureCode}</code></pre>
        </div>
      </div>
    </div>

    <div class="modal-section">
      <h3><span class="icon">💡</span> Key Insight</h3>
      <div class="info-card" style="background: var(--accent-green-glow);">
        <div class="value green" style="font-weight: 500; line-height: 1.6;">${
          exploit.keyInsight
        }</div>
      </div>
    </div>
  `

  overlay.classList.add('active')
  document.body.style.overflow = 'hidden'
}

function closeModal() {
  const overlay = document.getElementById('modal-overlay')
  if (overlay) {
    overlay.classList.remove('active')
    document.body.style.overflow = ''
  }
}

renderApp()
