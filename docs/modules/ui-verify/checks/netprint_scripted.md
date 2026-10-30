# `network_print_reaches_the_printer`

Drives File ▸ Print ▸ Send to network printer… on `fixtures/four-pages.pdf`
against a fake IPP printer the check serves on `127.0.0.1` (a free port). The
address reaches the window through `PDFCER_DIAG_NETPRINT_URI`, so nothing is
typed.

## What the fake printer says

- To Get-Printer-Attributes: `document-format-supported = application/pdf`,
  `finishings-supported = [3, 20]` (none, staple top-left),
  `output-bin-supported = top`, a make-and-model.
- To Print-Job: status `successful-ok`, `job-id = 7`.

Responses are hand-encoded (RFC 8010 §3) so the oracle does not share the
engine's encoder with the code under test.

## Oracles

1. `netprint-connected` carries `pdf=1 staples=1`: the offer was read
   through the engine's finishing check, not assumed.
2. The `netprint.staple` region is declared: a control is drawn exactly
   when the printer lists the option (R9).
3. `netprint-sent` carries `job_id=Some(7)`: the receipt is the printer's.
4. The fake printer received a request with operation `0x0002`
   (Print-Job) whose body contains `%PDF-`: the document itself crossed the
   wire, not only the attributes.

## What it cannot see

A real printer's behaviour after it accepts the job. `ipps://` (TLS) is not
driven; it is the engine's `pdfcer_fetch::post_ipp`.
