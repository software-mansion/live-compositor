import Link from '@docusaurus/Link';
import React, { useState } from 'react';

export default function RTCOnBanner() {
  const [show, setShow] = useState<boolean>(true);

  return show ? (
    <div
      className={
        'fixed bottom-0 left-0 ' +
        'grid ' +
        'grid-cols-[auto_32px] grid-rows-[auto_auto_auto] [column-gap:16px] [row-gap:16px] ' +
        "[grid-template-areas:'logo_close'_'text_close'_'button_button'] " +
        'sm:grid-cols-[max-content_auto_32px] sm:grid-rows-[auto_auto] sm:[column-gap:16px] sm:[row-gap:16px] ' +
        "sm:[grid-template-areas:'logo_text_close'_'button_button_button'] " +
        'lg:grid-cols-[max-content_max-content_auto_max-content_max-content] lg:grid-rows-[auto] lg:[column-gap:16px] ' +
        'lg:[grid-template-areas:"text_logo_decoration_button_close"] ' +
        'lg:h-[85px] w-screen px-6 py-6 lg:px-10 lg:py-0 ' +
        'border-t-2 border-t-solid border-t-white ' +
        'bg-gradient-to-r from-[#EA336F] to-[#4A01A7] ' +
        'text-white font-raleway'
      }>
      <div className="[grid-area:text] self-center flex flex-col items-start gap-4 ">
        <span className="uppercase text-base/6 lg:text-[22px]/9">
          We’re organizing a multimedia conference
        </span>
      </div>
      <div className="[grid-area:logo] self-center flex flex-col items-start">
        <img src={'/img/rtcon/logo.svg'} alt="RTCON logo" width="118" height="25" />
      </div>
      <div className="[grid-area:decoration] self-center hidden lg:flex flex-row justify-start h-[40px] max-w-[max(0px,calc((100%_-_40px)*999))]">
        <img src={'/img/rtcon/decoration.svg'} alt="RTCON decoration" width="40" height="48" />
      </div>
      <Link
        className="[grid-area:button] self-center max-w-[332px]"
        href="https://rtcon.live/"
        target="_blank"
        rel="noopener noreferrer">
        <div className="flex flex-row flex-nowrap px-6 py-1 gap-3 items-center justify-center border-2 border-solid border-white bg-white/20">
          <span className="uppercase font-semibold not-italic text-[18px]/6 text-white">
            Find out more
          </span>
          <img src={'/img/rtcon/arrowright.svg'} alt="Arrow right" width="42" height="42" />
        </div>
      </Link>
      <div className="[grid-area:close] lg:self-center flex flex-col items-center gap-4">
        <button onClick={() => setShow(false)} className="h-8 border-0 bg-transparent">
          <img src={'/img/rtcon/cross.svg'} alt="Cross" width="32" height="32" />
        </button>
      </div>
    </div>
  ) : (
    <></>
  );
}
