
	<div class="header">
		<p><slot /></p>
		<hr />
	</div>

    <style>
	.header p {
		margin-top: 0.3rem;
		margin-bottom: 0;
		padding-left: 0.3rem;
		font-size: 0.7rem;
		color: var(--foreground-3);
	}

	.header hr {
		margin: 0.2rem;
		border-color: var(--foreground-5);
	}

    </style>