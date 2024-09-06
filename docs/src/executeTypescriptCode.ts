import { transpileModule } from 'typescript';
import * as Babel from '@babel/standalone';
import { filter, isNonNull, map, pipe } from 'remeda';
import type { TraverseOptions } from '@babel/traverse';
import type TemplateGenerator from '@babel/template';
import { tsCompilerOptions } from './monacoEditorConfig';

const template = (Babel as unknown as { packages: { template: typeof TemplateGenerator } }).packages
  .template;

const staticToDynamicImports = {
  visitor: {
    ImportDeclaration(path) {
      const moduleName = path.node.source.value;

      const imports = pipe(
        path.node.specifiers,
        map(imp => {
          if (imp.type === 'ImportDefaultSpecifier') {
            return ['default', imp.local.name] as const;
          }

          if (imp.type === 'ImportSpecifier') {
            return [
              imp.imported.type === 'Identifier' ? imp.imported.name : imp.imported.value,
              imp.local.name,
            ] as const;
          }

          // Ignoring namespace imports
          return null;
        }),
        filter(isNonNull)
      );

      path.replaceWith(
        template.statement.ast(
          `const { ${imports
            .map(imp => (imp[0] === imp[1] ? imp[0] : `${imp[0]}: ${imp[1]}`))
            .join(',')} } = await _import('${moduleName}');`
        )
      );
    },
  } satisfies TraverseOptions,
};

const MAX_ITERATIONS = 2000;

/**
 * from https://github.com/facebook/react/blob/d906de7f602df810c38aa622c83023228b047db6/scripts/babel/transform-prevent-infinite-loops.js
 */
const preventInfiniteLoops = ({ types: t, template }: any) => {
  const buildGuard = template(`
    if (ITERATOR++ > MAX_ITERATIONS) {
      throw new RangeError(
        'Potential infinite loop: exceeded ' +
        MAX_ITERATIONS +
        ' iterations.'
      );
    }
  `);

  return {
    visitor: {
      'WhileStatement|ForStatement|DoWhileStatement': (path: any) => {
        const iterator = path.scope.parent.generateUidIdentifier('loopIt');
        const iteratorInit = t.numericLiteral(0);
        path.scope.parent.push({
          id: iterator,
          init: iteratorInit,
        });
        const guard = buildGuard({
          ITERATOR: iterator,
          MAX_ITERATIONS: t.numericLiteral(MAX_ITERATIONS),
        });

        if (!path.get('body').isBlockStatement()) {
          const statement = path.get('body').node;
          path.get('body').replaceWith(t.blockStatement([guard, statement]));
        } else {
          path.get('body').unshiftContainer('body', guard);
        }
      },
    },
  };
};

export default async function executeTypescriptCode(code: string) {
  try {
    const _import = async (moduleKey: string) => {
      if (moduleKey === 'react') {
        return await import('react');
      }
      if (moduleKey === 'react-dom') {
        return await import('react-dom');
      }
      if (moduleKey === 'live-compositor') {
        return await import('live-compositor');
      }
      throw new Error(`Module ${moduleKey} is not available in the sandbox.`);
    };
    const jsCode = transpileModule(code, {
      compilerOptions: tsCompilerOptions(),
    }).outputText;

    const transformedCode =
      Babel.transform(jsCode, {
        compact: false,
        retainLines: true,
        plugins: [staticToDynamicImports, preventInfiniteLoops],
      }).code ?? jsCode;

    const mod = Function(`
        return async (_import) => {
        ${transformedCode}
        };
        `);

    // Running the code
    await mod()(_import);
  } catch (error) {
    throw new Error(error.message);
  }
}
