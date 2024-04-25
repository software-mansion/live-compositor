import clsx from 'clsx';
import {
  FaChrome,
  FaCode,
  FaFile,
  FaFont,
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
import WebGpuLogoDark from '@site/static/img/webgpu-dark.svg';
import WebGpuLogoLight from '@site/static/img/webgpu-light.svg';
import { useColorMode } from '@docusaurus/theme-common';

function HomepageHeader() {
  return (
    <header className={clsx('hero hero--primary', styles.heroBanner)}>
      <div className="container">
        <div className="row">
          <div className="col col--2"></div>
          <div className="col col--8">
            <div className="container">
              <Heading as="h2" className="hero__title">
                The best solution for programmatic live video compositing.
              </Heading>
              <p>
                Process live video streams in real-time. Apply effects, combine videos, or add
                transitions to produce any output you want.
              </p>
              <div className="row">
                <Link
                  className="button button--secondary button--lg col col-6 margin--sm"
                  to="/docs/intro">
                  Docs
                </Link>
                <Link
                  className="button button--secondary button--outline button--lg col col-6 margin--sm"
                  to="https://github.com/membraneframework/live_compositor">
                  View on GitHub
                </Link>
              </div>
            </div>
          </div>
        </div>
      </div>
    </header>
  );
}

function ProsCards() {
  const wgpuLink = (
    <Link href="https://github.com/gfx-rs/wgpu">
      <code>wgpu</code>
    </Link>
  );
  return (
    <div className="container">
      <div className="row">
        <ProsCard title="Simple Declarative API" icon={FaCode}>
          <p className="padding--md">
            Simple Web-like component tree. Easy to pick up for anyone familiar with web
            development.
          </p>
        </ProsCard>
        <ProsCard title="Language agnostic" icon={FaLink}>
          <p className="padding--md">
            The entire configuration process happens over HTTP API, and streams are delivered over
            RTP. You can leverage technology of your choice and easily integrate it with your own
            solutions.
          </p>
        </ProsCard>
        <ProsCard title="Hardware accelerated" icon={FaMicrochip}>
          <p className="padding--md">
            Rendering is GPU accelerated using {wgpuLink} library, which implements API based on the
            WebGPU standard. Parts of the standard, like the ability to run WGSL shaders, are
            directly accessible via Live Compositor API.
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
    <div className={clsx('card', styles.card)}>
      <div style={{ display: 'flex', justifyContent: 'center' }}>
        <Icon className={styles.icon} />
      </div>
      <div className="card__header">
        <Heading as="h3" style={{ textAlign: 'center' }}>
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
  if (props.inverted) {
    return (
      <div className="row margin-right--lg margin-left--lg" style={{ alignItems: 'center' }}>
        {props.image}
        {text}
      </div>
    );
  } else {
    return (
      <div className="row margin-right--lg margin-left--lg" style={{ alignItems: 'center' }}>
        {text}
        {props.image}
      </div>
    );
  }
}

function Features() {
  const { colorMode } = useColorMode();
  const imageTextRendering = (
    <div className="row margin--lg">
      <div>
        <FaImage style={{ fontSize: 80, marginBottom: 30 }} />
      </div>
      <div>
        <FaFont style={{ fontSize: 80, marginTop: 30 }} />
      </div>
    </div>
  );
  const wgpuLogo =
    colorMode === 'dark' ? (
      <WebGpuLogoDark className="margin--lg" width={100} height={100} />
    ) : (
      <WebGpuLogoLight className="margin--lg" width={100} height={100} />
    );

  return (
    <div className="container margin-top--lg margin-bottom--md">
      <Heading as="h2" className="margin-bottom--md">
        Features
      </Heading>

      <Feature
        text="WebGPU APIs"
        secondaryText="Utilize existing WebGPU-based components or craft your own WGSL shader to achieve any desired effect and directly leverage GPU parallel processing capabilities."
        image={wgpuLogo}
      />
      <Feature
        text="Real-time processing"
        secondaryText="Process live video streams in real-time with low latency. Blazingly fast processing makes video conferencing, live-streaming, and broadcasting an everyday bread and butter for Live Compositor."
        image={<MdLiveTv className="margin--lg" style={{ fontSize: 100 }} />}
        inverted
      />
      <Feature
        text="Static content"
        secondaryText="Render text and static images onto your output streams or pass them to other components for further processing."
        image={imageTextRendering}
      />
      <Feature
        text="Web rendering"
        inverted
        secondaryText="You can render any website and combine it with video streams or other elements using the Chromium browser embedded inside the compositor."
        image={<FaChrome className="margin--lg" style={{ fontSize: 100 }} />}
      />
    </div>
  );
}

function IntegrationMembranePlugin() {
  return (
    <div className="row" style={{ alignItems: 'center' }}>
      <MembraneLogo height={220} width={220} className="margin--lg" />
      <div className="card col">
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
            <Link href="docs/get-started/membrane">Get started with Membrane plugin</Link>
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
  );
}

function Integrations() {
  return (
    <div className="container margin-top--lg margin-bottom--md">
      <Heading as="h2" className="margin-bottom--md">
        Integrations
      </Heading>
      <IntegrationMembranePlugin />
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
      <ProsCards />
      <Features />
      <Integrations />
    </Layout>
  );
}
