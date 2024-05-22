import clsx from 'clsx';
import {
  FaBook,
  FaChrome,
  FaCode,
  FaFile,
  FaGithub,
  FaImage,
  FaLink,
  FaMicrochip,
} from 'react-icons/fa6';
import { MdLiveTv } from 'react-icons/md';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import Heading from '@theme/Heading';

import styles from './index.module.css';
import { PropsWithChildren } from 'react';
import { IconType } from 'react-icons';
import MembraneLogo from '@site/static/img/membrane-logo.svg';
import SwmLogo from '@site/static/img/swm-logo.svg';
import WebGpuLogoDark from '@site/static/img/webgpu-dark.svg';
import WebGpuLogoLight from '@site/static/img/webgpu-light.svg';
import { useColorMode } from '@docusaurus/theme-common';
import TypewriterComponent from 'typewriter-effect';

function HomepageHeader() {
  return (
    <div className={clsx('container', styles.firstSection)}>
      <div className="row">
        <div className="col col--6">
          <div className={styles.shadow}></div>
          <div className="container">
            <Heading as="h1" style={{ fontSize: 55 }}>
              <span className="text--primary">Mix video and audio</span>
              <TypewriterComponent
                options={{
                  strings: ['in real-time.', 'using code.', 'with low-latency.'],
                  autoStart: true,
                  loop: true,
                  deleteSpeed: 30,
                  delay: 80,
                }}
              />
            </Heading>
            <p className={styles.sectionSubheading}>
              Open-source media server for real-time, low-latency, programmable video and audio
              mixing.
            </p>
            <div className="row margin-bottom--md">
              <Link
                className={clsx(
                  'button button--primary button--lg col col-6 margin--sm',
                  styles.hoverPrimary
                )}
                to="/docs/intro">
                <FaBook style={{ marginRight: 3 }} />
                Docs
              </Link>
              <Link
                className={clsx(
                  'button button--secondary button--outline button--lg col col-6 margin--sm',
                  styles.hoverSecondary
                )}
                to="https://github.com/membraneframework/live_compositor">
                <FaGithub style={{ marginRight: 5 }} />
                View on GitHub
              </Link>
            </div>
          </div>
        </div>
        <div className="col col--6">
          <div
            style={{
              borderRadius: '30px',
              overflow: 'hidden',
            }}>
            <video
              loop
              autoPlay
              muted
              src="https://github.com/membraneframework/live_compositor/assets/104033489/e6f5ba7c-ab05-4935-a42a-bc28c42fc895"
              poster="/img/demo_poster.png"
              style={{ width: '100%', display: 'block' }}
            />
          </div>
        </div>
      </div>
    </div>
  );
}

function ProsCards() {
  return (
    <div className="container margin-top--md">
      <Heading as="h1" className="margin-bottom--md text--center text--primary">
        Vision
      </Heading>
      <p className={clsx('text--center', styles.sectionSubheading)}>
        Make mixing live streams as simple as building a website.
      </p>
      <div className="row">
        <ProsCard title="Simple Declarative API" icon={FaCode}>
          <p className="padding--md">
            Simple Web-like component tree. Easy to pick up for anyone familiar with web
            development.
          </p>
        </ProsCard>
        <ProsCard title="Language agnostic" icon={FaLink}>
          <p className="padding--md">
            You can leverage tech stack of your choice and easily integrate it with your own
            solutions.
          </p>
        </ProsCard>
        <ProsCard title="Hardware accelerated" icon={FaMicrochip}>
          <p className="padding--md">
            Rendering is GPU accelerated using wgpu library, which implements API based on the
            WebGPU standard.
          </p>
        </ProsCard>
      </div>
    </div>
  );
}

type ProsCardProps = {
  title: string;
  icon: IconType;
};

function ProsCard(props: PropsWithChildren<ProsCardProps>) {
  const Icon = props.icon;
  return (
    <div className={clsx('card', styles.card, styles.hoverPrimary)}>
      <div style={{ display: 'flex', justifyContent: 'center' }}>
        <Icon className={styles.icon} />
      </div>
      <div className="card__header">
        <Heading as="h2" style={{ textAlign: 'center' }}>
          {props.title}
        </Heading>
      </div>
      <div className="card__body">{props.children}</div>
    </div>
  );
}

type FeatureProps = {
  text: string;
  secondaryText?: string;
  image: any;
  inverted?: boolean;
};

function Feature(props: FeatureProps) {
  const text = (
    <div className="col">
      <Heading as="h2" className="margin-top--sm">
        {props.text}
      </Heading>
      <p>{props.secondaryText}</p>
    </div>
  );
  return (
    <div className="row margin-right--lg margin-left--lg" style={{ alignItems: 'center' }}>
      {props.image}
      {text}
    </div>
  );
}

function Features() {
  const { colorMode } = useColorMode();

  const wgpuLogo =
    colorMode === 'dark' ? (
      <WebGpuLogoDark className={styles.featureIcon} width={80} height={80} />
    ) : (
      <WebGpuLogoLight className={styles.featureIcon} width={80} height={80} />
    );

  return (
    <div className="container margin-top--lg margin-bottom--md">
      <Heading as="h1" className="margin-bottom--md text--center text--primary">
        Features
      </Heading>
      <p className={clsx('text--center', styles.sectionSubheading)}>
        Simple, powerful, fast. Pick three.
      </p>

      <Feature
        text="WebGPU APIs"
        secondaryText="Utilize existing WebGPU-based components or craft your own WGSL shader to achieve any desired effect and directly leverage GPU parallel processing capabilities."
        image={wgpuLogo}
      />
      <Feature
        text="Real-time processing"
        secondaryText="Process live video streams in real-time with low latency. Blazingly fast processing makes video conferencing, live-streaming, and broadcasting an everyday bread and butter for Live Compositor."
        image={<MdLiveTv className={styles.featureIcon} style={{ fontSize: 80 }} />}
        inverted
      />
      <Feature
        text="Static content"
        secondaryText="Render text and static images onto your output streams or pass them to other components for further processing."
        image={<FaImage className={styles.featureIcon} style={{ fontSize: 80 }} />}
      />
      <Feature
        text="Web rendering"
        inverted
        secondaryText="You can render any website and combine it with video streams or other elements using the Chromium browser embedded inside the compositor."
        image={<FaChrome className={styles.featureIcon} style={{ fontSize: 80 }} />}
      />
    </div>
  );
}

function IntegrationMembranePlugin() {
  return (
    <div className="row" style={{ justifyContent: 'center' }}>
      <MembraneLogo
        width={200}
        height={200}
        className="margin--lg"
        style={{ alignSelf: 'center' }}
      />
      <div className="col">
        <div className="card">
          <div className="card__header">
            <Heading as="h3" className="margin-top--sm">
              Membrane plugin
            </Heading>
          </div>
          <div className="card__body container">
            <p>
              Membrane is a developer-friendly multimedia framework for Elixir. You can easily add
              video composing functionality into your multimedia pipeline using Membrane Live
              Compositor Plugin.
            </p>
            <p>For more, see:</p>
            <p>
              <FaFile style={{ fontSize: 15, marginRight: 7 }} />
              Documentation -{' '}
              <Link href="/docs/intro#membrane-framework-plugin">
                Get started with Membrane plugin
              </Link>
            </p>
            <p>
              <FaGithub style={{ fontSize: 15, marginRight: 7 }} />
              GitHub repository -{' '}
              <Link href="https://github.com/membraneframework/membrane_live_compositor_plugin">
                membraneframework/membrane_live_compositor_plugin
              </Link>
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}

function Integrations() {
  return (
    <div className="container">
      <Heading as="h1" className="margin-bottom--md text--center text--primary">
        Integrations
      </Heading>
      <IntegrationMembranePlugin />
    </div>
  );
}

function ContactUs() {
  return (
    <div className="container margin-bottom--lg">
      <div className="card container">
        <div className="row" style={{ justifyContent: 'center' }}>
          <SwmLogo width={220} className="margin--lg" style={{ alignSelf: 'center' }} />
          <div className="col">
            <div className="card__header margin-top--md">
              <Heading as="h1">
                <span className="text--primary">Contact</span> us
              </Heading>
            </div>
            <div className="card__body">
              <p className={styles.sectionSubheading}>
                LiveCompositor is free and open-source. It's created by Software Mansion - a
                software company that is specialized in building tools for developers. At Software
                Mansion, we work on multiple multimedia projects, like Membrane Framework, Elixir
                WebRTC, FishJam, and more. We also work on custom solutions for clients. Contact us
                and create something together.
              </p>
              <div className="row" style={{ justifyContent: 'end' }}>
                <Link
                  className={clsx(
                    'button',
                    'button--primary',
                    'button--lg',
                    'margin--sm',
                    styles.contactButton,
                    styles.hoverPrimary
                  )}
                  to="https://membrane.stream/contact">
                  Contact us
                </Link>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default function Home(): JSX.Element {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout
      title={siteConfig.title}
      description="Tool for real-time video processing / transforming / composing">
      <HomepageHeader />
      <div className={styles.sectionSeparator} />
      <ProsCards />
      <div className={styles.sectionSeparator} />
      <Features />
      <div className={styles.sectionSeparator} />
      <Integrations />
      <div className={styles.sectionSeparator} />
      <ContactUs />
    </Layout>
  );
}
